//! # Localhost Plugin Module
//!
//! Configures and creates the Tauri localhost plugin with CORS headers for
//! Service Workers and an OTLP proxy for build-baked telemetry.

use std::{
	io::{Read, Write},
	net::TcpStream,
	time::Duration,
};

use tauri::plugin::TauriPlugin;

/// OTLP collector host:port. OTELBridge.ts sends to `/v1/traces` (same-origin),
/// this proxy forwards to the real collector via raw TCP. Zero CORS issues.
const OTLP_HOST:&str = "127.0.0.1:4318";

/// Forward a JSON body to the OTLP collector via raw HTTP/1.1 POST.
/// Returns true if the collector accepted (2xx), false otherwise.
fn ProxyToOTLP(Body:&[u8]) -> bool {
	let Ok(mut Stream) = TcpStream::connect_timeout(&OTLP_HOST.parse().unwrap(), Duration::from_millis(500)) else {
		return false;
	};

	let _ = Stream.set_write_timeout(Some(Duration::from_millis(500)));
	let _ = Stream.set_read_timeout(Some(Duration::from_millis(500)));

	let Request = format!(
		"POST /v1/traces HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: \
		 close\r\n\r\n",
		OTLP_HOST,
		Body.len(),
	);

	if Stream.write_all(Request.as_bytes()).is_err() {
		return false;
	}
	if Stream.write_all(Body).is_err() {
		return false;
	}

	let mut ResponseBuffer = [0u8; 32];
	let _ = Stream.read(&mut ResponseBuffer);
	// Check for "HTTP/1.1 2" - any 2xx status
	ResponseBuffer.starts_with(b"HTTP/1.1 2") || ResponseBuffer.starts_with(b"HTTP/1.0 2")
}

/// Creates and configures the localhost plugin with CORS headers preconfigured.
///
/// # CORS Configuration
///
/// - Access-Control-Allow-Origin: * (allows all origins)
/// - Access-Control-Allow-Methods: GET, POST, OPTIONS, HEAD
/// - Access-Control-Allow-Headers: Content-Type, Authorization, Origin, Accept
///
/// # OTLP Proxy
///
/// Requests to `/v1/traces` are forwarded to the local OTLP collector
/// (Jaeger, OTEL Collector, etc.) so OTELBridge.ts can send telemetry
/// without cross-origin issues. Uses raw TCP - no extra HTTP client dependency.
pub fn LocalhostPlugin<R:tauri::Runtime>(ServerPort:u16) -> TauriPlugin<R> {
	tauri_plugin_localhost::Builder::new(ServerPort)
		.on_request(|Request, Response| {
			// CORS headers for Service Workers and frontend integration
			Response.add_header("Access-Control-Allow-Origin", "*");
			Response.add_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS, HEAD");
			Response.add_header("Access-Control-Allow-Headers", "Content-Type, Authorization, Origin, Accept");

			// OTLP proxy: forward /v1/traces to the local collector
			let Url = Request.url();
			if Url.contains("/v1/traces") {
				Response.set_handled(true);
				let Body = Request.body();
				if ProxyToOTLP(Body) {
					Response.set_status(200);
					Response.add_header("Content-Type", "application/json");
				} else {
					// Collector not running - 204 silently.
					// OTELBridge stops retrying after first failure.
					Response.set_status(204);
				}
			}
		})
		.build()
}
