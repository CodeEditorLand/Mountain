//! Forwards an HTTP request to a local service over a raw TCP
//! connection and returns `(status, body, headers)`.

use std::collections::HashMap;

use tauri::http::{Method, request::Request};

use super::ParseHttpResponse;
use crate::dev_log;

/// Forward an HTTP request to a local service
///
/// # Parameters
///
/// - `url`: The full URL to forward to (e.g., "http://127.0.0.1:8080/path")
/// - `request`: The original Tauri request
/// - `method`: The HTTP method to use
///
/// # Returns
///
/// A Tauri response with status, headers, and body from the forwarded request
pub(crate) fn Fn(
	url:&str,

	request:&Request<Vec<u8>>,

	method:Method,
) -> Result<(u16, Vec<u8>, HashMap<String, String>), String> {
	// Parse URL to get host and path
	let parsed_url = url.parse::<http::uri::Uri>().map_err(|e| format!("Invalid URL: {}", e))?;

	// Extract host, port, and path as owned strings to satisfy 'static lifetime
	let host = parsed_url.host().ok_or("No host in URL")?.to_string();

	let port = parsed_url.port_u16().unwrap_or(80);

	let path = parsed_url
		.path_and_query()
		.map(|p| p.as_str().to_string())
		.unwrap_or_else(|| "/".to_string());

	let addr = format!("{}:{}", host, port);

	dev_log!("lifecycle", "[Scheme] Connecting to {} at {}", url, addr);

	// Clone request body and headers for use in thread
	let body = request.body().clone();

	let headers:Vec<(String, String)> = request
		.headers()
		.iter()
		.filter_map(|(name, value)| {
			let header_name = name.as_str().to_lowercase();

			let hop_by_hop_headers = [
				"connection",
				"keep-alive",
				"proxy-authenticate",
				"proxy-authorization",
				"te",
				"trailers",
				"transfer-encoding",
				"upgrade",
			];

			if !hop_by_hop_headers.contains(&header_name.as_str()) {
				value.to_str().ok().map(|v| (name.as_str().to_string(), v.to_string()))
			} else {
				None
			}
		})
		.collect();

	// Use tokio runtime to make the request
	let result = std::thread::spawn(move || {
		let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;

		rt.block_on(async {
			use tokio::{
				io::{AsyncReadExt, AsyncWriteExt},
				net::TcpStream,
			};

			// Connect to the service
			let mut stream = TcpStream::connect(&addr)
				.await
				.map_err(|e| format!("Failed to connect: {}", e))?;

			// Build HTTP request
			let mut request_str = format!("{} {} HTTP/1.1\r\nHost: {}\r\n", method.as_str(), path, host);

			// Add headers
			for (name, value) in &headers {
				request_str.push_str(&format!("{}: {}\r\n", name, value));
			}

			// Add Content-Length if there's a body
			if !body.is_empty() {
				request_str.push_str(&format!("Content-Length: {}\r\n", body.len()));
			}

			request_str.push_str("\r\n");

			// Send request
			stream
				.write_all(request_str.as_bytes())
				.await
				.map_err(|e| format!("Failed to write request: {}", e))?;

			if !body.is_empty() {
				stream
					.write_all(&body)
					.await
					.map_err(|e| format!("Failed to write body: {}", e))?;
			}

			// Read response
			let mut buffer = Vec::new();

			let mut temp_buf = [0u8; 8192];

			loop {
				let n = stream
					.read(&mut temp_buf)
					.await
					.map_err(|e| format!("Failed to read response: {}", e))?;

				if n == 0 {
					break;
				}

				buffer.extend_from_slice(&temp_buf[..n]);

				// Check if we've read the full response (simple check for content-length or end
				// of headers)
				if buffer.len() > 1024 * 1024 {
					// Limit to 1MB
					dev_log!("lifecycle", "warn: [Scheme] Response too large, truncating");

					break;
				}

				// Simple heuristic: if we have a full HTTP response with Content-Length, check
				// if we've read everything
				if let Some(headers_end) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
					let headers = String::from_utf8_lossy(&buffer[..headers_end]);

					if let Some(cl_line) = headers.lines().find(|l| l.to_lowercase().starts_with("content-length:")) {
						if let Ok(cl) = cl_line.trim_start_matches("content-length:").trim().parse::<usize>() {
							let body_expected = headers_end + 4 + cl;

							if buffer.len() >= body_expected {
								break;
							}
						}
					} else if !headers.contains("Transfer-Encoding: chunked") {
						// No Content-Length and not chunked, assume complete if connection closes
						continue;
					}
				}
			}

			// Parse response - pass raw bytes so binary bodies (PNG, etc.)
			// are never corrupted by UTF-8 lossy conversion.
			ParseHttpResponse::Fn(&buffer)
		})
	})
	.join()
	.map_err(|e| format!("Thread panicked: {:?}", e))?;

	result
}
