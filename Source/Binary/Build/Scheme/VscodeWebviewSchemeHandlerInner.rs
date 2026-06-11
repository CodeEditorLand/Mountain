//! Resolves a `vscode-webview://` request to the webview shell files
//! under `vs/workbench/contrib/webview/browser/pre/`, trying the
//! embedded asset resolver first and the dev-mode filesystem root
//! second. Called inside the panic guard in
//! `Scheme::VscodeWebviewSchemeHandler`.

use tauri::http::response::{Builder, Response};

use super::{MimeFromExtension, build_error_response};
use crate::dev_log;

pub(crate) fn Fn<R:tauri::Runtime>(
	AppHandle:&tauri::AppHandle<R>,

	Request:&tauri::http::request::Request<Vec<u8>>,
) -> Response<Vec<u8>> {
	let Uri = Request.uri().to_string();

	dev_log!("scheme-assets", "[LandFix:VscodeWebview] Request: {}", Uri);

	// `vscode-webview://<authority>/<path>?<query>`. We only care about
	// `<path>` - authority is per-instance noise, querystring is the
	// `id`/`parentId`/`extensionId`/etc that `index.html` reads via
	// `URLSearchParams` (we don't touch it).
	let After = match Uri.strip_prefix("vscode-webview://") {
		Some(Rest) => Rest,

		None => {
			return build_error_response(400, "vscode-webview scheme without prefix");
		},
	};

	let PathStart = match After.find('/') {
		Some(Index) => Index + 1,

		None => {
			return build_error_response(400, "vscode-webview URI missing path component");
		},
	};

	let PathPlusQuery = &After[PathStart..];

	// Trim the querystring + fragment - filesystem doesn't care.
	let CleanPath:&str = PathPlusQuery
		.split_once(|C:char| C == '?' || C == '#')
		.map(|(Path, _)| Path)
		.unwrap_or(PathPlusQuery);

	// Reject path-traversal attempts. The webview shell is a static
	// three-file directory; anything containing `..` or hitting
	// outside `pre/` is hostile or a bug.
	if CleanPath.is_empty() || CleanPath.contains("..") {
		return build_error_response(404, "vscode-webview path empty or traversal");
	}

	let ResolvedPath = format!("Static/Application/vs/workbench/contrib/webview/browser/pre/{}", CleanPath);

	dev_log!(
		"scheme-assets",
		"[LandFix:VscodeWebview] resolve {} -> {}",
		CleanPath,
		ResolvedPath
	);

	// Try the embedded asset resolver first (release / packaged builds
	// where `Sky/Target/Static/Application/` is bundled into Mountain's
	// binary). Falls through to the filesystem fallback below for
	// debug-electron-bundled, where assets ship next to Mountain.
	if let Some(Asset) = AppHandle.asset_resolver().get(ResolvedPath.clone()) {
		let Mime = MimeFromExtension::Fn(&ResolvedPath);

		dev_log!(
			"scheme-assets",
			"[LandFix:VscodeWebview] serve embedded {} ({}, {} bytes)",
			ResolvedPath,
			Mime,
			Asset.bytes.len()
		);

		return Builder::new()
			.status(200)
			.header("Content-Type", Mime)
			.header("Access-Control-Allow-Origin", "*")
			.header("Cross-Origin-Embedder-Policy", "require-corp")
			.header("Cross-Origin-Resource-Policy", "cross-origin")
			.header("Cache-Control", "no-cache")
			.body(Asset.bytes.to_vec())
			.unwrap_or_else(|_| build_error_response(500, "Failed to build response"));
	}

	// Filesystem fallback for dev mode. `ApplicationRoot` is set by
	// `Binary/Main/AppLifecycle.rs` to the resolved `Sky/Target/`
	// directory at startup so we can read the same `pre/` files the
	// embedded resolver would have served.
	let StaticRoot = crate::IPC::WindServiceHandlers::Utilities::ApplicationRoot::Get::Fn();

	if let Some(Root) = StaticRoot {
		let FilesystemPath = std::path::Path::new(&Root).join(&ResolvedPath);

		if FilesystemPath.exists() && FilesystemPath.is_file() {
			match std::fs::read(&FilesystemPath) {
				Ok(Bytes) => {
					let Mime = MimeFromExtension::Fn(&ResolvedPath);

					dev_log!(
						"scheme-assets",
						"[LandFix:VscodeWebview] serve filesystem {} ({}, {} bytes)",
						FilesystemPath.display(),
						Mime,
						Bytes.len()
					);

					return Builder::new()
						.status(200)
						.header("Content-Type", Mime)
						.header("Access-Control-Allow-Origin", "*")
						.header("Cross-Origin-Embedder-Policy", "require-corp")
						.header("Cross-Origin-Resource-Policy", "cross-origin")
						.header("Cache-Control", "no-cache")
						.body(Bytes)
						.unwrap_or_else(|_| build_error_response(500, "Failed to build response"));
				},

				Err(Error) => {
					dev_log!(
						"lifecycle",
						"warn: [LandFix:VscodeWebview] Failed to read {}: {}",
						FilesystemPath.display(),
						Error
					);
				},
			}
		}
	}

	dev_log!(
		"lifecycle",
		"warn: [LandFix:VscodeWebview] Not found: {} (resolved: {})",
		Uri,
		ResolvedPath
	);

	build_error_response(404, &format!("Not Found: {}", ResolvedPath))
}
