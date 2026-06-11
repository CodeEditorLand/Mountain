//! Parses a raw HTTP response into `(status, body, headers)`.
//! Operates on raw bytes so binary bodies are never corrupted.

use std::collections::HashMap;

/// Parse a raw HTTP response into (status, body, headers).
/// Operates on raw bytes so binary bodies (PNG, JPEG, WASM, etc.) are never
/// corrupted by UTF-8 lossy conversion. Only the headers portion (which is
/// always ASCII) is decoded as UTF-8.
pub(crate) fn Fn(response:&[u8]) -> Result<(u16, Vec<u8>, HashMap<String, String>), String> {
	let headers_end = response
		.windows(4)
		.position(|w| w == b"\r\n\r\n")
		.ok_or("Invalid HTTP response: no headers/body separator")?;

	let headers_str =
		std::str::from_utf8(&response[..headers_end]).map_err(|e| format!("Invalid UTF-8 in HTTP headers: {}", e))?;

	let body = response[headers_end + 4..].to_vec();

	// Parse status line
	let mut lines = headers_str.lines();

	let status_line = lines.next().ok_or("Invalid HTTP response: no status line")?;

	// Parse status code (e.g., "HTTP/1.1 200 OK" -> 200)
	let status = status_line
		.split_whitespace()
		.nth(1)
		.and_then(|s| s.parse::<u16>().ok())
		.ok_or_else(|| format!("Invalid status line: {}", status_line))?;

	// Parse headers
	let mut headers = HashMap::new();

	for line in lines {
		if let Some((name, value)) = line.split_once(':') {
			headers.insert(name.trim().to_lowercase(), value.trim().to_string());
		}
	}

	Ok((status, body, headers))
}
