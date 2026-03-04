//! # Scheme Handler Module
//!
//! Provides custom URI scheme handlers for Tauri webview isolation.
//!
//! ## RESPONSIBILITIES
//!
//! - Handle `land://` custom protocol requests
//! - Routing to local HTTP services via ServiceRegistry
//! - Forward HTTP requests (GET, POST, PUT, DELETE, PATCH) to local services
//! - Set appropriate CORS headers for webview isolation
//! - Handle CORS preflight requests (OPTIONS method)
//! - Implement basic caching for static assets
//! - Handle health checks and error scenarios
//!
//! ## ARCHITECTURAL ROLE
//!
//! The Scheme module provides protocol-level isolation and routing for
//! webviews:
//!
//! ```text
//! land://code.editor.land/path ──► ServiceRegistry ──► http://127.0.0.1:PORT/path
//!                                       │                        │
//!                                       ▼                        ▼
//!                               CORS Headers Set          Local Service
//!                                                            Response
//! ```
//!
//! ## SECURITY
//!
//! - All responses include Access-Control-Allow-Origin: land://code.editor.land
//! - Content-Type preserved from local service response
//! - CORS headers set appropriately for cross-origin requests
//! - Request validation and sanitization

use std::{collections::HashMap, sync::RwLock};

use tauri::http::{
	Method,
	request::Request,
	response::{Builder, Response},
};
use log::{debug, error, info, warn};

use super::ServiceRegistry::ServiceRegistry;

// Global service registry (will be initialized in Tauri setup)
static SERVICE_REGISTRY:RwLock<Option<ServiceRegistry>> = RwLock::new(None);

/// Initialize the global service registry
///
/// This must be called once during application setup before any land://
/// requests.
pub fn init_service_registry(registry:ServiceRegistry) {
	let mut registry_lock = SERVICE_REGISTRY.write().unwrap();
	*registry_lock = Some(registry);
}

/// Get a reference to the global service registry
///
/// Returns None if not initialized (should not happen in normal operation).
///
/// # Safety
/// This function uses an unsafe block to get a static reference to the
/// service registry. This is safe because:
/// 1. The SERVICE_REGISTRY is a static RwLock that lives for the entire program
/// 2. We only write to it during initialization (before any land:// requests)
/// 3. After initialization, we only read from it
/// 4. The RwLock guarantees thread-safe access
fn get_service_registry() -> Option<ServiceRegistry> {
let guard = SERVICE_REGISTRY.read().ok()?;
guard.clone()
}

/// DNS port managed state structure
///
/// This struct holds the DNS server port number and is managed by Tauri
/// as application state, making it accessible to Tauri commands.
#[derive(Clone, Debug)]
pub struct DnsPort(pub u16);

/// Cache entry for static asset caching
#[derive(Clone)]
struct CacheEntry {
	/// Cached response bytes
	body:Vec<u8>,
	/// Content-Type header value
	content_type:String,
	/// Cache-Control header value
	cache_control:String,
	/// ETag for conditional requests
	etag:Option<String>,
	/// Last-Modified timestamp
	last_modified:Option<String>,
}

/// Simple in-memory cache for static assets
///
/// Uses a HashMap to store cached responses by URL path.
/// This is a basic implementation that could be enhanced with:
/// - TTL-based expiration
/// - LRU eviction when cache is full
/// - Size limits
static CACHE:RwLock<Option<HashMap<String, CacheEntry>>> = RwLock::new(None);

/// Initialize the static asset cache
fn init_cache() {
	let mut cache = CACHE.write().unwrap();
	if cache.is_none() {
		*cache = Some(HashMap::new());
	}
}

/// Get a cached response if available
fn get_cached(path:&str) -> Option<CacheEntry> {
	let cache = CACHE.read().unwrap();
	cache.as_ref()?.get(path).cloned()
}

/// Store a response in the cache
fn set_cached(path:&str, entry:CacheEntry) {
	let mut cache = CACHE.write().unwrap();
	if let Some(cache) = cache.as_mut() {
		cache.insert(path.to_string(), entry);
	}
}

/// Check if a path should be cached
///
/// Returns true for CSS, JS, images, fonts, and other static assets.
fn should_cache(path:&str) -> bool {
	let path_lower = path.to_lowercase();
	path_lower.ends_with(".css")
		|| path_lower.ends_with(".js")
		|| path_lower.ends_with(".png")
		|| path_lower.ends_with(".jpg")
		|| path_lower.ends_with(".jpeg")
		|| path_lower.ends_with(".gif")
		|| path_lower.ends_with(".svg")
		|| path_lower.ends_with(".woff")
		|| path_lower.ends_with(".woff2")
		|| path_lower.ends_with(".ttf")
		|| path_lower.ends_with(".eot")
		|| path_lower.ends_with(".ico")
}

/// Parse a land:// URI to extract domain and path
///
/// # Parameters
///
/// - `uri`: The land:// URI (e.g., "land://code.editor.land/path/to/resource")
///
/// # Returns
///
/// A tuple of (domain, path) where:
/// - domain: "code.editor.land"
/// - path: "/path/to/resource"
///
/// # Example
///
/// ```rust
/// let (domain, path) = parse_land_uri("land://code.editor.land/api/status");
/// assert_eq!(domain, "code.editor.land");
/// assert_eq!(path, "/api/status");
/// ```
fn parse_land_uri(uri:&str) -> Result<(String, String), String> {
	// Remove the land:// prefix
	let without_scheme = uri
		.strip_prefix("land://")
		.ok_or_else(|| format!("Invalid land:// URI: {}", uri))?;

	// Split into domain and path
	let parts:Vec<&str> = without_scheme.splitn(2, '/').collect();

	let domain = parts.get(0).ok_or_else(|| format!("No domain in URI: {}", uri))?.to_string();

	let path = if parts.len() > 1 { format!("/{}", parts[1]) } else { "/".to_string() };

	debug!("[Scheme] Parsed URI: {} -> domain={}, path={}", uri, domain, path);
	Ok((domain, path))
}

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
fn forward_http_request(
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

	debug!("[Scheme] Connecting to {} at {}", url, addr);

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
					warn!("[Scheme] Response too large, truncating");
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

			// Parse response
			let response_str = String::from_utf8_lossy(&buffer);
			parse_http_response(&response_str)
		})
	})
	.join()
	.map_err(|e| format!("Thread panicked: {:?}", e))?;

	result
}

/// Parse an HTTP response string into status, body, and headers
fn parse_http_response(response:&str) -> Result<(u16, Vec<u8>, HashMap<String, String>), String> {
	// Split headers and body
	let headers_end = response
		.find("\r\n\r\n")
		.ok_or("Invalid HTTP response: no headers/body separator")?;

	let headers_str = &response[..headers_end];
	let body = response[headers_end + 4..].as_bytes().to_vec();

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
///
/// # Example
///
/// ```rust
/// tauri::Builder::default()
/// 	.register_uri_scheme_protocol("land", |_app, request| land_scheme_handler(request))
/// ```
pub fn land_scheme_handler(request:&Request<Vec<u8>>) -> Response<Vec<u8>> {
	// Initialize cache on first request
	init_cache();

	// Get URI
	let uri = request.uri().to_string();
	debug!("[Scheme] Handling land:// request: {}", uri);

	// Parse URI to extract domain and path
	let (domain, path) = match parse_land_uri(&uri) {
		Ok(result) => result,
		Err(e) => {
			error!("[Scheme] Failed to parse URI: {}", e);
			return build_error_response(400, &format!("Bad Request: {}", e));
		},
	};

	// Handle CORS preflight requests
	if request.method() == Method::OPTIONS {
		debug!("[Scheme] Handling CORS preflight request");
		return build_cors_preflight_response();
	}

	// Check cache for static assets
	if should_cache(&path) {
		if let Some(cached) = get_cached(&path) {
			debug!("[Scheme] Cache hit for: {}", path);
			return build_cached_response(cached);
		}
	}

	// Look up service in registry
	let registry = match get_service_registry() {
		Some(r) => r,
		None => {
			error!("[Scheme] Service registry not initialized");
			return build_error_response(503, "Service Unavailable: Registry not initialized");
		},
	};

	let service = match registry.lookup(&domain) {
		Some(s) => s,
		None => {
			warn!("[Scheme] Service not found: {}", domain);
			return build_error_response(404, &format!("Not Found: Service {} not registered", domain));
		},
	};

	// Build local service URL
	let local_url = format!("http://127.0.0.1:{}{}", service.port, path);

	debug!(
		"[Scheme] Routing {} {} to local service at {}",
		request.method(),
		uri,
		local_url
	);

	// Forward request to local service
	let result = forward_http_request(&local_url, request, request.method().clone());

	match result {
		Ok((status, body, headers)) => {
			// Clone body before using it
			let body_bytes = body.clone();

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
				debug!("[Scheme] Cached response for: {}", path);
			}

			response.unwrap_or_else(|_| build_error_response(500, "Internal Server Error"))
		},
		Err(e) => {
			error!("[Scheme] Failed to forward request: {}", e);
			build_error_response(503, &format!("Service Unavailable: {}", e))
		},
	}
}

/// Build an error response with CORS headers
fn build_error_response(status:u16, message:&str) -> Response<Vec<u8>> {
	let body = serde_json::json!({
		"error": message,
		"status": status
	});

	Builder::new()
		.status(status)
		.header("Content-Type", "application/json")
		.header("Access-Control-Allow-Origin", "land://code.editor.land")
		.body(serde_json::to_vec(&body).unwrap_or_default())
		.unwrap_or_else(|_| Builder::new().status(500).body(Vec::new()).unwrap())
}

/// Build a CORS preflight response
fn build_cors_preflight_response() -> Response<Vec<u8>> {
	Builder::new()
		.status(204)
		.header("Access-Control-Allow-Origin", "land://code.editor.land")
		.header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, PATCH, OPTIONS")
		.header("Access-Control-Allow-Headers", "Content-Type, Authorization, X-Requested-With")
		.header("Access-Control-Max-Age", "86400")
		.body(Vec::new())
		.unwrap()
}

/// Build a response from cached data
fn build_cached_response(entry:CacheEntry) -> Response<Vec<u8>> {
	let mut builder = Builder::new()
		.status(200)
		.header("Content-Type", &entry.content_type)
		.header("Access-Control-Allow-Origin", "land://code.editor.land")
		.header("Cache-Control", &entry.cache_control);

	if let Some(etag) = &entry.etag {
		builder = builder.header("ETag", etag);
	}

	if let Some(last_modified) = &entry.last_modified {
		builder = builder.header("Last-Modified", last_modified);
	}

	builder
		.body(entry.body)
		.unwrap_or_else(|_| build_error_response(500, "Internal Server Error"))
}

/// Register a service with the land:// scheme
///
/// This helper function makes it easy to register local services.
///
/// # Parameters
///
/// - `name`: Domain name (e.g., "code.editor.land")
/// - `port`: Local port where the service is listening
pub fn register_land_service(name:&str, port:u16) {
	let registry = get_service_registry().expect("Service registry not initialized. Call init_service_registry first.");
	registry.register(name.to_string(), port, Some("/health".to_string()));
	info!("[Scheme] Registered service: {} -> {}", name, port);
}

/// Get the port for a registered service
///
/// # Parameters
///
/// - `name`: Domain name to look up
///
/// # Returns
///
/// - `Some(port)` if service is registered
/// - `None` if service not found
pub fn get_land_port(name:&str) -> Option<u16> {
	let registry = get_service_registry()?;
	registry.lookup(name).map(|s| s.port)
}

/// Handles `land://` custom protocol requests asynchronously
///
/// This is the asynchronous version of `land_scheme_handler` that uses
/// Tauri's `UriSchemeResponder` to respond asynchronously, allowing the
/// request processing to happen in a separate thread.
///
/// This is the recommended handler for production use as it provides better
/// performance and doesn't block the main thread.
///
/// # Parameters
///
/// - `_ctx`: The URI scheme context (not used in current implementation)
/// - `request`: The incoming webview request with URI path and headers
/// - `responder`: The responder to send the response back asynchronously
///
/// # Platform Support
///
/// - **macOS, Linux**: Uses `land://localhost/` as Origin
/// - **Windows**: Uses `http://land.localhost/` as Origin by default
///
/// # Example
///
/// ```rust
/// tauri::Builder::default()
/// 	.register_asynchronous_uri_scheme_protocol("land", |_ctx, request, responder| {
/// 		land_scheme_handler_async(_ctx, request, responder)
/// 	})
/// ```
///
/// Note: This implementation uses thread spawning as a workaround since
/// Tauri 2.x's async scheme handler API requires specific runtime setup.
/// The thread-based approach works correctly and is production-ready.
pub fn land_scheme_handler_async<R:tauri::Runtime>(
	_ctx:tauri::UriSchemeContext<'_, R>,
	request:tauri::http::request::Request<Vec<u8>>,
	responder:tauri::UriSchemeResponder,
) {
	// Spawn a new thread to handle the request asynchronously
	std::thread::spawn(move || {
		let response = land_scheme_handler(&request);
		responder.respond(response);
	});
}

/// Get the appropriate Access-Control-Allow-Origin header for the current
/// platform
///
/// Tauri uses different origins for custom URI schemes on different platforms:
/// - macOS, Linux: land://localhost/
/// - Windows: http://land.localhost/
///
/// Returns a comma-separated list of origins to support all platforms.
fn get_cors_origins() -> &'static str {
	// Support both macOS/Linux (land://localhost) and Windows (http://land.localhost)
	"land://localhost, http://land.localhost, land://code.editor.land"
}

/// Initializes the scheme handler module
///
/// This is a placeholder function that can be used for any future
/// initialization logic needed by the scheme handler.
#[inline]
pub fn Scheme() {}
