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

use std::{
	collections::HashMap,
	panic::{AssertUnwindSafe, catch_unwind},
	sync::RwLock,
};

use tauri::http::{
	request::Request,
	response::{Builder, Response},
};

use super::ServiceRegistry::ServiceRegistry;
use crate::dev_log;

pub mod ForwardHttpRequest;

pub mod LandSchemeHandler;

pub mod MimeFromExtension;

pub mod ParseHttpResponse;

pub mod VscodeFileSchemeHandlerInner;

pub mod VscodeWebviewSchemeHandlerInner;

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

	dev_log!("lifecycle", "[Scheme] Parsed URI: {} -> domain={}, path={}", uri, domain, path);

	Ok((domain, path))
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
/// 	.register_uri_scheme_protocol("fiddee", |_app, request| fiddee_scheme_handler(request))
/// ```
pub fn land_scheme_handler(request:&Request<Vec<u8>>) -> Response<Vec<u8>> { LandSchemeHandler::Fn(request) }

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

	dev_log!("lifecycle", "[Scheme] Registered service: {} -> {}", name, port);
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
/// 	.register_asynchronous_uri_scheme_protocol("fiddee", |_ctx, request, responder| {
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
/// - Windows: <http://land.localhost/>
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

// ==========================================================================
// vscode-file:// Protocol Handler
// ==========================================================================

/// Handles `vscode-file://` custom protocol requests.
///
/// VS Code's Electron workbench computes asset URLs as:
///   `vscode-file://vscode-app/{appRoot}/out/vs/workbench/...`
///
/// This handler maps those URLs to the embedded frontend assets
/// served from the `frontendDist` directory (`../Sky/Target`).
///
/// # URL Mapping
///
/// ```text
/// vscode-file://vscode-app/Static/Application/vs/workbench/foo.js
///                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
///                          This path maps to Sky/Target/Static/Application/vs/workbench/foo.js
/// ```
///
/// The `/out/` prefix that the workbench appends is stripped if present,
/// since our assets live at `/Static/Application/vs/` not
/// `/Static/Application/out/vs/`.
///
/// # Parameters
///
/// - `AppHandle`: Tauri AppHandle for resolving the frontend dist path
/// - `Request`: The incoming request
///
/// # Returns
///
/// Response with file contents and correct MIME type, or 404
pub fn VscodeFileSchemeHandler<R:tauri::Runtime>(
	AppHandle:&tauri::AppHandle<R>,

	Request:&tauri::http::request::Request<Vec<u8>>,
) -> Response<Vec<u8>> {
	// The scheme handler runs inside the wkwebview URL loading code
	// (Objective-C FFI). A panic here crosses an `extern "C"` boundary
	// that cannot unwind - the process aborts immediately. Catch the
	// panic so a bad mmap or MIME bug returns a 500 instead of taking
	// the whole editor down.
	let Result = catch_unwind(AssertUnwindSafe(|| VscodeFileSchemeHandlerInner::Fn(AppHandle, Request)));

	match Result {
		Ok(Response) => Response,

		Err(Panic) => {
			let Info = if let Some(Text) = Panic.downcast_ref::<&str>() {
				Text.to_string()
			} else if let Some(Text) = Panic.downcast_ref::<String>() {
				Text.clone()
			} else {
				"unknown panic".to_string()
			};

			dev_log!(
				"lifecycle",
				"error: [LandFix:VscodeFile] caught panic in scheme handler: {}",
				Info
			);

			build_error_response(500, &format!("Internal Server Error (caught panic: {})", Info))
		},
	}
}

/// Custom URI scheme handler for `vscode-webview://` requests.
///
/// VS Code's `WebviewElement` (used by every extension webview - Roo
/// Code, Claude, GitLens, custom-editor providers) wraps the inner
/// extension HTML in an `<iframe>` whose `src` is
/// `vscode-webview://<authority>/index.html?...`. The `<authority>` is
/// a per-instance random base32 string. The authority is irrelevant to
/// the bytes served - all that matters is the path component, which
/// always resolves under
/// `vs/workbench/contrib/webview/browser/pre/`.
///
/// In stock Electron VS Code, `app.protocol.registerStreamProtocol(
/// 'vscode-webview', ...)` serves this directory. Under Tauri 2.x +
/// WKWebView, `register_asynchronous_uri_scheme_protocol("vscode-webview",
/// ...)` installs an equivalent `WKURLSchemeHandler`. Without this handler,
/// every extension that uses `webviewView` / `WebviewPanel` /
/// `CustomEditor` lands the inner iframe at a `vscode-webview://...`
/// URL the WKWebView can't resolve, the iframe stays blank, and the
/// extension surface is dead.
///
/// Three resources live under `pre/`:
///   - `index.html`        - the webview shell that bridges `postMessage`
///     between workbench host and inner extension HTML
///   - `service-worker.js` - registered by `index.html` to intercept
///     `vscode-webview-resource` requests for extension-shipped assets
///   - `fake.html`         - sandbox stub used as a placeholder before
///     extension HTML arrives via postMessage
///
/// Anything else (querystrings, extra path segments, GUID-like
/// authorities) is silently dropped; the extension's actual content
/// gets piped in via the `swMessage` channel after `index.html` boots,
/// not through this scheme handler.
///
/// # Parameters
///
/// - `AppHandle`: Tauri AppHandle for resolving the embedded asset resolver and
///   the dev-mode `Static/Application/` filesystem fallback (same chain as
///   `VscodeFileSchemeHandler`).
/// - `Request`: The incoming request - typically a `GET` for one of the three
///   pre-baked files.
///
/// # Returns
///
/// A `Response<Vec<u8>>` carrying:
///   - `200 OK` with the file bytes + correct MIME (`text/html` /
///     `application/javascript`) when found, or
///   - `404 Not Found` when the resolved path falls outside the `pre/`
///     directory or the asset isn't shipped.
///
/// CORS headers are permissive (`*`) to match the workbench host's
/// `vscode-webview-resource:` traffic, which round-trips through the
/// service worker registered by `index.html`.
pub fn VscodeWebviewSchemeHandler<R:tauri::Runtime>(
	AppHandle:&tauri::AppHandle<R>,

	Request:&tauri::http::request::Request<Vec<u8>>,
) -> Response<Vec<u8>> {
	let Result = catch_unwind(AssertUnwindSafe(|| VscodeWebviewSchemeHandlerInner::Fn(AppHandle, Request)));

	match Result {
		Ok(Response) => Response,

		Err(Panic) => {
			let Info = if let Some(Text) = Panic.downcast_ref::<&str>() {
				Text.to_string()
			} else if let Some(Text) = Panic.downcast_ref::<String>() {
				Text.clone()
			} else {
				"unknown panic".to_string()
			};

			dev_log!(
				"lifecycle",
				"error: [LandFix:VscodeWebview] caught panic in scheme handler: {}",
				Info
			);

			build_error_response(500, &format!("Internal Server Error (caught panic: {})", Info))
		},
	}
}
