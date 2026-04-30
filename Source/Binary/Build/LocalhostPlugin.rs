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
///
/// Currently unused - the OTLP proxy path requires `Response::set_handled` /
/// `set_status` / `Request::body()` from a patched fork of
/// `tauri-plugin-localhost`. After resetting the vendored copy to upstream
/// (`Dependency/Tauri/Dependency/PluginsWorkspace/plugins/localhost`),
/// those methods are gone;
#[allow(dead_code)]
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
/// Fonts are listed explicitly. WKWebView is strict about font MIME types -
/// when the asset resolver falls back to `application/octet-stream` for a
/// `.ttf` (which `infer` does on some macOS versions because TrueType has
/// no magic header), the browser silently refuses to use the font and the
/// workbench renders icons as blank squares with no console error. The
/// codicon font is the visible symptom; KaTeX and Seti fonts under
/// `/Static/Application/extensions/...` follow the same path.
///
/// Returns `None` for unknown extensions so the plugin's `asset.mime_type`
/// fallback still applies (images, WASM, etc.).
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
		"ttf" => Some("font/ttf"),
		"otf" => Some("font/otf"),
		"woff" => Some("font/woff"),
		"woff2" => Some("font/woff2"),
		"eot" => Some("application/vnd.ms-fontobject"),
		_ => None,
	}
}

/// Forward a JSON body to the OTLP collector via raw HTTP/1.1 POST.
/// Returns true if the collector accepted (2xx), false otherwise.
///
/// See `OTLP_HOST` for why this is currently unused.
#[allow(dead_code)]
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
			// CORS headers for Service Workers and frontend integration.
			Response.add_header("Access-Control-Allow-Origin", "*");
			Response.add_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS, HEAD");
			Response.add_header("Access-Control-Allow-Headers", "Content-Type, Authorization, Origin, Accept");

			let Url = Request.url();

			// LAND-PATCH B1.X: the upstream tauri-plugin-localhost `Response`
			// only exposes `add_header` - no `set_handled` / `set_status`,
			// and `Request` exposes only `url()` (no `body()`). Mountain's
			// previous OTLP proxy + status override depended on a patched
			// fork. After resetting the vendored copy to upstream those
			// methods are gone. The OTLP proxy is moved to the dev OTEL
			// collector (run separately from Tauri); the status override
			// is no longer needed because the upstream plugin always emits
			// 200 OK on a successful asset hit and the asset resolver's
			// 404 path is sufficient for the un-mocked case.
			//
			// To restore the OTLP-proxy / status-override path, patch the
			// vendored `tauri-plugin-localhost` (`Dependency/Tauri/
			// Dependency/PluginsWorkspace/plugins/localhost/src/lib.rs`)
			// to add `Response::set_handled(bool)`, `Response::set_status(
			// u16)`, and `Request::body() -> &[u8]`.

			// Pre-set the correct `Content-Type` for known asset extensions.
			// The upstream plugin sets `Content-Type` from `asset.mime_type`
			// before invoking the on_request callback, but the user-set
			// value via `add_header` overrides it (HashMap insert).
			if let Some(Mime) = MimeFromUrl(Url) {
				Response.add_header("Content-Type", Mime);
			}
		})
		.build()
}
