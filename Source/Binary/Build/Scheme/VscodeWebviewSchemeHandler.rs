//! `Scheme::VscodeWebviewSchemeHandler`

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
pub fn Fn<R:tauri::Runtime>(
	AppHandle:&tauri::AppHandle<R>,

	Request:&tauri::http::request::Request<Vec<u8>>,
) -> Response<Vec<u8>> {
	let Result = catch_unwind(AssertUnwindSafe(|| _VscodeWebviewSchemeHandler(AppHandle, Request)));

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

			BuildErrorResponse(500, &format!("Internal Server Error (caught panic: {})", Info))
		},
	}
}
