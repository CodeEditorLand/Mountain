//! Resolves a `vscode-file://` request against embedded assets, the
//! mmap asset cache, absolute OS paths, and the dev-mode `Sky/Target`
//! filesystem root. Called inside the panic guard in
//! `Scheme::VscodeFileSchemeHandler`.

use tauri::http::response::{Builder, Response};

use super::{MimeFromExtension, build_error_response};
use crate::dev_log;

pub(crate) fn Fn<R:tauri::Runtime>(
	AppHandle:&tauri::AppHandle<R>,

	Request:&tauri::http::request::Request<Vec<u8>>,
) -> Response<Vec<u8>> {
	let Uri = Request.uri().to_string();

	// Per-asset-request line - every `<img src="vscode-file://...">` +
	// worker / wasm / font in the workbench fires through here. The
	// `scheme-assets` line below (opt-in tag) already captures the
	// same data; duplicating under `lifecycle` at the default level
	// just floods the log.
	dev_log!("scheme-assets", "[LandFix:VscodeFile] Request: {}", Uri);

	dev_log!("scheme-assets", "[SchemeAssets] request uri={}", Uri);

	// Extract path from: vscode-file://<authority>/<path>
	//
	// The canonical workbench-side authority is `vscode-app` (used by
	// `FileAccess.uriToBrowserUri` for ALL workbench resources). But
	// `WebviewImplementation::asWebviewUri` rewrites local resource
	// URIs to use the extension's identifier as the authority - e.g.
	// `vscode-file://vscode.git/Volumes/.../extensions/git/media/icon.svg`.
	// The strip-prefix chain below covers both:
	//   1. Exact `vscode-app` authority (with or without trailing `/`)
	//   2. ANY other authority - we treat the post-authority path as the resource
	//      path and let the OS-absolute-root detection below serve it straight from
	//      disk. Without this fallback every extension-supplied webview asset
	//      (icons, scripts, stylesheets, fonts) returned 404 because the strip
	//      yielded `""` and the asset_resolver lookup ran with an empty key.
	let FilePath = Uri
		.strip_prefix("vscode-file://vscode-app/")
		.or_else(|| Uri.strip_prefix("vscode-file://vscode-app"))
		.or_else(|| {
			// Generic `vscode-file://<authority>/<path>` - skip past the
			// `vscode-file://` scheme + the authority's first `/`.
			let After = Uri.strip_prefix("vscode-file://")?;

			let SlashIdx = After.find('/')?;

			Some(&After[SlashIdx + 1..])
		})
		.unwrap_or("");

	// Strip /out/ prefix if present - our assets are at /Static/Application/vs/
	// not /Static/Application/out/vs/
	let CleanPath = if FilePath.starts_with("Static/Application//out/") {
		FilePath.replacen("Static/Application//out/", "Static/Application/", 1)
	} else if FilePath.starts_with("Static/Application/out/") {
		FilePath.replacen("Static/Application/out/", "Static/Application/", 1)
	} else {
		FilePath.to_string()
	};

	// VS Code's nodeModulesPath = 'vs/../../node_modules' resolves ../../ from
	// Static/Application/vs/ up to Static/. The browser canonicalizes this to
	// Static/node_modules/ but our files live at Static/Application/node_modules/.
	let CleanPath = if CleanPath.starts_with("Static/node_modules/") {
		CleanPath.replacen("Static/node_modules/", "Static/Application/node_modules/", 1)
	} else {
		CleanPath
	};

	// Strip `?<query>` and `#<fragment>` from the resolved path so
	// filesystem / asset-resolver lookups operate on a clean path
	// component. Roo's runtime sourcemap-probe (`vZt` in its bundle)
	// fetches `<src>?source-map=true` which would otherwise hit the
	// asset_resolver as a literal `index.js?source-map=true` filename
	// and either 404 or fall through to the SPA-fallback `index.html`
	// (5765 bytes served as `application/octet-stream`). With the
	// strip, `index.js?source-map=true` → `index.js`, which exists on
	// disk and serves correctly with the right MIME. Equivalent for
	// `#<fragment>`. Sourcemap-probe URLs that point to non-existent
	// suffixes (`index.map.json`, `index.sourcemap`) still 404
	// silently; that is the intended behavior of `vZt`'s preload list.
	let CleanPath = match CleanPath.split_once(['?', '#']) {
		Some((Before, _)) => Before.to_string(),

		None => CleanPath,
	};

	// P1.5 fix: DevTools fetches `*.js.map` for every bundled script it loads
	// to render pretty stack traces. Our `Static/Application/` tree ships the
	// JS files without their `.map` siblings (esbuild's `sourcemap:false` path)
	// so those requests always 404. Short-circuit here with a clean
	// `204 No Content` - Chromium treats 204 as "no map available" and moves
	// on silently, avoiding both the noisy stderr lines and the filesystem
	// stat round-trip per request.
	if CleanPath.ends_with(".map") {
		return Builder::new()
			.status(204)
			.header("Access-Control-Allow-Origin", "*")
			.header("Cross-Origin-Resource-Policy", "cross-origin")
			.body(Vec::new())
			.unwrap_or_else(|_| build_error_response(500, "Failed to build response"));
	}

	// CSS-as-JS shim: when a `.css` URL is requested through
	// `vscode-file://` (which happens for any unstripped raw `import
	// "./foo.css"` that VS Code's bundle still contains after
	// `workbench.js` switches `_VSCODE_FILE_ROOT` to the custom
	// scheme), the browser would refuse the response with
	// `'text/css' is not a valid JavaScript MIME type`. Service
	// Workers can't intercept custom-scheme requests, so we inline
	// the same JS shim the Worker SW emits on the localhost path:
	// invoke `_LOAD_CSS_WORKER` against the localhost-form path and
	// export an empty default. The SW + `<link>` fast-path then
	// loads the actual CSS bytes from `/Static/Application/...`.
	//
	// CRITICAL gate: only apply the shim for paths under
	// `Static/Application/` (i.e. workbench-internal CSS imports
	// that survive bundling as `import "./foo.css"`). Extension-
	// contributed CSS lives in absolute filesystem paths
	// (`Users/...`, `Volumes/...`, `Library/...`, etc.) and reaches
	// `vscode-file://` via `WebviewImplementation::asWebviewUri`.
	// Those `.css` files MUST be served as real `text/css` from
	// disk (the IsAbsoluteOSPath fallback below handles them) -
	// returning the JS shim instead silently breaks every
	// extension webview-ui that bundles its own stylesheet
	// (Roo: `webview-ui/build/assets/index.css`, Claude, GitLens,
	// Continue, etc. all use Vite/webpack and ship CSS bundles).
	// Without this gate the iframe loads no styles and the panel
	// renders as a transparent overlay over the workbench - the
	// classic "blank webview" symptom.
	if CleanPath.ends_with(".css") && CleanPath.starts_with("Static/Application/") {
		let LocalPath = format!("/Static/Application/{}", CleanPath.trim_start_matches("Static/Application/"));

		let Body = format!("globalThis._LOAD_CSS_WORKER?.({:?}); export default {{}};", LocalPath);

		dev_log!(
			"scheme-assets",
			"[LandFix:VscodeFile] css-shim {} -> _LOAD_CSS_WORKER({})",
			CleanPath,
			LocalPath
		);

		return Builder::new()
			.status(200)
			.header("Content-Type", "application/javascript; charset=utf-8")
			.header("Access-Control-Allow-Origin", "*")
			.header("Cross-Origin-Resource-Policy", "cross-origin")
			.header("Cross-Origin-Embedder-Policy", "require-corp")
			.header("Cache-Control", "public, max-age=31536000, immutable")
			.body(Body.into_bytes())
			.unwrap_or_else(|_| build_error_response(500, "Failed to build response"));
	}

	// Icon themes, grammars and other extension-contributed assets generate
	// URIs like `vscode-file://vscode-app/Volumes/<vol>/.../seti.woff` after
	// `FileAccess.uriToBrowserUri` rewrites a plain `file:///Volumes/...`
	// extension path. The authority `vscode-app` is followed directly by the
	// absolute filesystem path (sans leading `/`). Detect the well-known macOS /
	// Linux absolute-path roots and serve straight from disk instead of trying
	// to resolve them against `Sky/Target/` (where they do not exist).
	let IsAbsoluteOSPath = [
		"Volumes/",
		"Users/",
		"Library/",
		"System/",
		"Applications/",
		"private/",
		"tmp/",
		"var/",
		"etc/",
		"opt/",
		"home/",
		"usr/",
		"srv/",
		"mnt/",
		"root/",
	]
	.iter()
	.any(|Prefix| CleanPath.starts_with(Prefix));

	if IsAbsoluteOSPath {
		let AbsolutePath = format!("/{}", CleanPath);

		let FilesystemPath = std::path::Path::new(&AbsolutePath);

		dev_log!(
			"scheme-assets",
			"[LandFix:VscodeFile] os-abs candidate {} (exists={}, is_file={})",
			AbsolutePath,
			FilesystemPath.exists(),
			FilesystemPath.is_file()
		);

		if FilesystemPath.exists() && FilesystemPath.is_file() {
			// LAND-PATCH B7.P01: route through the mmap cache. First
			// hit on a path mmaps the file; subsequent hits are
			// wait-free DashMap reads. Brotli sibling (`<file>.br`)
			// is auto-discovered and served when the request offers
			// `Accept-Encoding: br`.
			match ::Cache::AssetMemoryMap::LoadOrInsert::Fn(FilesystemPath) {
				Ok(Entry) => {
					let AcceptsBrotli = Request
						.headers()
						.get("accept-encoding")
						.and_then(|V| V.to_str().ok())
						.map(|S| S.contains("br"))
						.unwrap_or(false);

					let (Body, Encoding):(Vec<u8>, Option<&str>) = if AcceptsBrotli {
						match Entry.AsBrotliSlice() {
							Some(Slice) => (Slice.to_vec(), Some("br")),

							None => (Entry.AsSlice().to_vec(), None),
						}
					} else {
						(Entry.AsSlice().to_vec(), None)
					};

					dev_log!(
						"scheme-assets",
						"[LandFix:VscodeFile] os-abs served {} ({}, {} bytes, encoding={:?})",
						AbsolutePath,
						Entry.Mime,
						Body.len(),
						Encoding
					);

					// `Cross-Origin-Resource-Policy: cross-origin` lets the
					// COEP-isolated webview iframe (which Mountain serves
					// from the `vscode-webview://` scheme with
					// `Cross-Origin-Embedder-Policy: require-corp`) load
					// these assets via `<script src=…>` / `<link href=…>`.
					// Without it WebKit refuses to expose the response to
					// the embedder document and the extension's React
					// bundle / CSS / fonts come up as cross-origin
					// resource-policy blocks.
					let mut B = Builder::new()
						.status(200)
						.header("Content-Type", Entry.Mime)
						.header("Access-Control-Allow-Origin", "*")
						.header("Cross-Origin-Resource-Policy", "cross-origin")
						.header("Cross-Origin-Embedder-Policy", "require-corp")
						.header("Cache-Control", "public, max-age=3600");

					if let Some(Enc) = Encoding {
						B = B.header("Content-Encoding", Enc);
					}

					return B
						.body(Body)
						.unwrap_or_else(|_| build_error_response(500, "Failed to build response"));
				},

				Err(Error) => {
					dev_log!(
						"lifecycle",
						"warn: [LandFix:VscodeFile] os-abs mmap failure {}: {}",
						AbsolutePath,
						Error
					);
				},
			}
		} else {
			dev_log!("lifecycle", "warn: [LandFix:VscodeFile] os-abs not on disk: {}", AbsolutePath);
		}
	}

	dev_log!("lifecycle", "[LandFix:VscodeFile] Resolved path: {}", CleanPath);

	// Resolve against the frontendDist directory
	// In production: embedded in the binary via asset_resolver
	// In debug: fall back to filesystem read from Sky/Target
	let AssetResult = AppHandle.asset_resolver().get(CleanPath.clone());

	if let Some(Asset) = AssetResult {
		let Mime = MimeFromExtension::Fn(&CleanPath);

		dev_log!(
			"lifecycle",
			"[LandFix:VscodeFile] Serving (embedded) {} ({}, {} bytes)",
			CleanPath,
			Mime,
			Asset.bytes.len()
		);

		dev_log!(
			"scheme-assets",
			"[SchemeAssets] serve source=embedded path={} mime={} bytes={}",
			CleanPath,
			Mime,
			Asset.bytes.len()
		);

		return Builder::new()
			.status(200)
			.header("Content-Type", Mime)
			.header("Access-Control-Allow-Origin", "*")
			.header("Cross-Origin-Resource-Policy", "cross-origin")
			.header("Cross-Origin-Embedder-Policy", "require-corp")
			.header("Cache-Control", "public, max-age=31536000, immutable")
			.body(Asset.bytes.to_vec())
			.unwrap_or_else(|_| build_error_response(500, "Failed to build response"));
	}

	// Fallback: read from filesystem (dev mode where assets aren't embedded)
	let StaticRoot = crate::IPC::WindServiceHandlers::Utilities::ApplicationRoot::Get::Fn();

	if let Some(Root) = StaticRoot {
		let FilesystemPath = std::path::Path::new(&Root).join(&CleanPath);

		if FilesystemPath.exists() && FilesystemPath.is_file() {
			// LAND-PATCH B7.P01: mmap-cache the StaticRoot fallback
			// path so dev-mode workbench reloads pay the syscall
			// once per asset for the entire session.
			match ::Cache::AssetMemoryMap::LoadOrInsert::Fn(&FilesystemPath) {
				Ok(Entry) => {
					let AcceptsBrotli = Request
						.headers()
						.get("accept-encoding")
						.and_then(|V| V.to_str().ok())
						.map(|S| S.contains("br"))
						.unwrap_or(false);

					let (Body, Encoding):(Vec<u8>, Option<&str>) = if AcceptsBrotli {
						match Entry.AsBrotliSlice() {
							Some(Slice) => (Slice.to_vec(), Some("br")),

							None => (Entry.AsSlice().to_vec(), None),
						}
					} else {
						(Entry.AsSlice().to_vec(), None)
					};

					dev_log!(
						"lifecycle",
						"[LandFix:VscodeFile] Serving (fs-mmap) {} ({}, {} bytes, encoding={:?})",
						CleanPath,
						Entry.Mime,
						Body.len(),
						Encoding
					);

					// `Cross-Origin-Resource-Policy: cross-origin` lets the
					// COEP-isolated webview iframe (which Mountain serves
					// from the `vscode-webview://` scheme with
					// `Cross-Origin-Embedder-Policy: require-corp`) load
					// these assets via `<script src=…>` / `<link href=…>`.
					// Without it WebKit refuses to expose the response to
					// the embedder document and the extension's React
					// bundle / CSS / fonts come up as cross-origin
					// resource-policy blocks.
					let mut B = Builder::new()
						.status(200)
						.header("Content-Type", Entry.Mime)
						.header("Access-Control-Allow-Origin", "*")
						.header("Cross-Origin-Resource-Policy", "cross-origin")
						.header("Cross-Origin-Embedder-Policy", "require-corp")
						.header("Cache-Control", "public, max-age=3600");

					if let Some(Enc) = Encoding {
						B = B.header("Content-Encoding", Enc);
					}

					return B
						.body(Body)
						.unwrap_or_else(|_| build_error_response(500, "Failed to build response"));
				},

				Err(Error) => {
					dev_log!(
						"lifecycle",
						"warn: [LandFix:VscodeFile] Failed to read {}: {}",
						FilesystemPath.display(),
						Error
					);
				},
			}
		}
	}

	dev_log!(
		"lifecycle",
		"warn: [LandFix:VscodeFile] Not found: {} (resolved: {})",
		Uri,
		CleanPath
	);

	build_error_response(404, &format!("Not Found: {}", CleanPath))
}
