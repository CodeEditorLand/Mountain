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

/// Resolve the correct `Content-Type` for a request URL by its file extension.
///
/// The vendored `tauri-plugin-localhost` asset resolver sometimes reports
/// `text/html` for disk-served `.js` / `.css` assets, which breaks module
/// loading in the webview (the browser refuses JS with `'text/html' is not a
/// valid JavaScript MIME type'`). By pre-setting `Content-Type` in
/// `on_request`, we guarantee the right MIME for the extensions the workbench
/// actually loads; the patched plugin keeps our value instead of overwriting.
///
/// Returns `None` for unknown extensions so the plugin's `asset.mime_type`
/// fallback still applies (e.g. images, fonts, WASM — the asset resolver
/// handles those correctly).
fn MimeFromUrl(Url:&str) -> Option<&'static str> {
	// Strip query string / fragment before extension match.
	let Path = Url.split(['?', '#']).next().unwrap_or(Url);

	let Extension = Path.rsplit('.').next()?.to_ascii_lowercase();

	match Extension.as_str() {
		"js" | "mjs" | "cjs" => Some("application/javascript; charset=utf-8"),
		"css" => Some("text/css; charset=utf-8"),
		"json" | "map" => Some("application/json; charset=utf-8"),
		"html" | "htm" => Some("text/html; charset=utf-8"),
		"svg" => Some("image/svg+xml"),
		"wasm" => Some("application/wasm"),
		"txt" => Some("text/plain; charset=utf-8"),
		_ => None,
	}
}

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

			let Url = Request.url();

			// OTLP proxy: forward /v1/traces to the local collector
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
				return;
			}

			// Pre-set the correct `Content-Type` for known asset extensions.
			// The patched localhost plugin only falls back to asset.mime_type
			// when this header is absent, so setting it here makes our value
			// authoritative. Fixes module-loading errors of the form
			// `'text/html' is not a valid JavaScript MIME type`.
			if let Some(Mime) = MimeFromUrl(Url) {
				Response.add_header("Content-Type", Mime);
			}
		})
		.build()
}
