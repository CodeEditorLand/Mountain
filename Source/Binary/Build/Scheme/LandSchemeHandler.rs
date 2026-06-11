//! Routes a `land://` request to the registered local HTTP service,
//! applying CORS headers, MIME-honesty on asset 404s, and the
//! static-asset cache.

use tauri::http::{
	Method,
	request::Request,
	response::{Builder, Response},
};

use super::{
	CacheEntry,
	ForwardHttpRequest,
	build_cached_response,
	build_cors_preflight_response,
	build_error_response,
	get_cached,
	get_service_registry,
	init_cache,
	parse_land_uri,
	set_cached,
	should_cache,
};
use crate::dev_log;

/// Handles `land://` custom protocol requests
///
/// This function is called by Tauri when a webview makes a request to the
/// `land://` protocol. It routes the request to local HTTP services via the
/// ServiceRegistry.
///
/// # Parameters
///
/// - `request`: The incoming webview request with URI path and headers
///
/// # Returns
///
/// A Tauri response with:
/// - Status code from local service (or error status)
/// - Headers from local service plus CORS headers
/// - Response body from local service (or error body)
///
/// # Implementation Details
///
/// 1. Parse the land:// URI to extract domain and path
/// 2. Look up the service in the ServiceRegistry
/// 3. Handle CORS preflight (OPTIONS) requests
/// 4. Check cache for static assets
/// 5. Forward the request to the local service
/// 6. Add CORS headers to the response
/// 7. Cache static assets for future requests
///
/// # Error Handling
///
/// - 400: Invalid URI format
/// - 404: Service not found in registry
/// - 503: Service unavailable / request failed
pub(crate) fn Fn(request:&Request<Vec<u8>>) -> Response<Vec<u8>> {
	// Initialize cache on first request
	init_cache();

	// Get URI
	let uri = request.uri().to_string();

	dev_log!("lifecycle", "[Scheme] Handling land:// request: {}", uri);

	// Parse URI to extract domain and path
	let (domain, path) = match parse_land_uri(&uri) {
		Ok(result) => result,

		Err(e) => {
			dev_log!("lifecycle", "error: [Scheme] Failed to parse URI: {}", e);

			return build_error_response(400, &format!("Bad Request: {}", e));
		},
	};

	// Handle CORS preflight requests
	if request.method() == Method::OPTIONS {
		dev_log!("lifecycle", "[Scheme] Handling CORS preflight request");

		return build_cors_preflight_response();
	}

	// Check cache for static assets
	if should_cache(&path) {
		if let Some(cached) = get_cached(&path) {
			dev_log!("lifecycle", "[Scheme] Cache hit for: {}", path);

			return build_cached_response(cached);
		}
	}

	// Look up service in registry
	let registry = match get_service_registry() {
		Some(r) => r,

		None => {
			dev_log!("lifecycle", "error: [Scheme] Service registry not initialized");

			return build_error_response(503, "Service Unavailable: Registry not initialized");
		},
	};

	let service = match registry.lookup(&domain) {
		Some(s) => s,

		None => {
			dev_log!("lifecycle", "warn: [Scheme] Service not found: {}", domain);

			return build_error_response(404, &format!("Not Found: Service {} not registered", domain));
		},
	};

	// Build local service URL
	let local_url = format!("http://127.0.0.1:{}{}", service.port, path);

	dev_log!(
		"lifecycle",
		"[Scheme] Routing {} {} to local service at {}",
		request.method(),
		uri,
		local_url
	);

	// Forward request to local service
	let result = ForwardHttpRequest::Fn(&local_url, request, request.method().clone());

	match result {
		Ok((status, body, headers)) => {
			// Clone body before using it
			let body_bytes = body.clone();

			// LAND-FIX B1.P1: MIME-honesty on 404. The localhost
			// server (or Astro/Vite dev page underneath) returns an
			// HTML body with `Content-Type: text/html` for any
			// missing path. The webview asks for `.js`/`.json`/`.css`
			// files; when it parses the HTML body as JS it crashes
			// with `SyntaxError: Unexpected token '<'` at column N -
			// the exact symptom reported in the release-electron-
			// bundled run. Rewrite the response to text/plain empty
			// body when the request was for a known asset extension
			// AND upstream returned non-2xx.
			let LowerPath = path.to_ascii_lowercase();

			let IsAssetRequest = LowerPath.ends_with(".js")
				|| LowerPath.ends_with(".mjs")
				|| LowerPath.ends_with(".cjs")
				|| LowerPath.ends_with(".json")
				|| LowerPath.ends_with(".map")
				|| LowerPath.ends_with(".css")
				|| LowerPath.ends_with(".wasm")
				|| LowerPath.ends_with(".svg")
				|| LowerPath.ends_with(".png")
				|| LowerPath.ends_with(".woff")
				|| LowerPath.ends_with(".woff2")
				|| LowerPath.ends_with(".ttf")
				|| LowerPath.ends_with(".otf");

			let UpstreamSaysHtml = headers
				.get("content-type")
				.map(|V| V.to_ascii_lowercase().contains("text/html"))
				.unwrap_or(false);

			if IsAssetRequest && (status == 404 || (status >= 400 && UpstreamSaysHtml)) {
				dev_log!(
					"scheme-assets",
					"[LandFix:Mime] swap HTML 404 → text/plain empty for asset path={} status={}",
					path,
					status
				);

				return Builder::new()
					.status(404)
					.header("Content-Type", "text/plain; charset=utf-8")
					.header("Access-Control-Allow-Origin", "land://code.editor.land")
					.body(Vec::<u8>::new())
					.unwrap_or_else(|_| build_error_response(500, "Failed to build 404 response"));
			}

			// Build response with CORS headers
			let mut response_builder = Builder::new()
				.status(status)
				.header("Access-Control-Allow-Origin", "land://code.editor.land")
				.header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, PATCH, OPTIONS")
				.header("Access-Control-Allow-Headers", "Content-Type, Authorization, X-Requested-With");

			// Add important headers from local service
			let important_headers = [
				"content-type",
				"content-length",
				"etag",
				"last-modified",
				"cache-control",
				"expires",
				"content-encoding",
				"content-disposition",
				"location",
			];

			for header_name in &important_headers {
				if let Some(value) = headers.get(*header_name) {
					response_builder = response_builder.header(*header_name, value);
				}
			}

			let response = response_builder.body(body_bytes);

			// Cache static assets
			if status == 200 && should_cache(&path) {
				let content_type = headers
					.get("content-type")
					.unwrap_or(&"application/octet-stream".to_string())
					.clone();

				let cache_control = headers
					.get("cache-control")
					.unwrap_or(&"public, max-age=3600".to_string())
					.clone();

				let etag = headers.get("etag").cloned();

				let last_modified = headers.get("last-modified").cloned();

				let entry = CacheEntry { body, content_type, cache_control, etag, last_modified };

				set_cached(&path, entry);

				dev_log!("lifecycle", "[Scheme] Cached response for: {}", path);
			}

			response.unwrap_or_else(|_| build_error_response(500, "Internal Server Error"))
		},

		Err(e) => {
			dev_log!("lifecycle", "error: [Scheme] Failed to forward request: {}", e);

			build_error_response(503, &format!("Service Unavailable: {}", e))
		},
	}
}
