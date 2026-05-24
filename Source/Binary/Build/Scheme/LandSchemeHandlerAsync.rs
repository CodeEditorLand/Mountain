//! `Scheme::LandSchemeHandlerAsync`

use std::{
	collections::HashMap,
	panic::{AssertUnwindSafe, catch_unwind},
	sync::RwLock,
};
use tauri::http::{
	Method,
	request::Request,
	response::{Builder, Response},
};
use super::ServiceRegistry::Struct;
use crate::dev_log;

static SERVICE_REGISTRY:RwLock<Option<ServiceRegistry>> = RwLock::new(None);
static CACHE:RwLock<Option<HashMap<String, CacheEntry>>> = RwLock::new(None);

/// Handles `land://` custom protocol requests asynchronously
///
/// This is the asynchronous version of `LandSchemeHandler` that uses
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
/// 		LandSchemeHandlerAsync(_ctx, request, responder)
/// 	}
