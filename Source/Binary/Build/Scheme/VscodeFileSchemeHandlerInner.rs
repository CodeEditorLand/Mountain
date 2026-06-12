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

	let CleanPath = NormalizePath(FilePath);

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

		dev_log!("scheme-assets", "[LandFix:VscodeFile] os-abs candidate {}", AbsolutePath);

		// LAND-PATCH B7.P01: route through the mmap cache. First
		// hit on a path mmaps the file; subsequent hits are
		// wait-free DashMap reads. Brotli sibling (`<file>.br`)
		// is auto-discovered and served when the request offers
		// `Accept-Encoding: br`. The open itself is the existence
		// probe - `NotFound` is the miss path, so no separate
		// `exists()` / `is_file()` stat() pair runs per request.
		match ::Cache::AssetMemoryMap::LoadOrInsert::Fn(FilesystemPath) {
			Ok(Entry) => {
				let CacheControl = CacheControlFor(&CleanPath);

				if IfNoneMatchSatisfied(Request, &Entry.ETag) {
					dev_log!("scheme-assets", "[LandFix:VscodeFile] os-abs 304 {}", AbsolutePath);

					return BuildNotModified(&Entry.ETag, CacheControl);
				}

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
					.header("ETag", Entry.ETag.as_str())
					.header("Cache-Control", CacheControl);

				if let Some(Enc) = Encoding {
					B = B.header("Content-Encoding", Enc);
				}

				return B
					.body(Body)
					.unwrap_or_else(|_| build_error_response(500, "Failed to build response"));
			},

			Err(Error) if Error.kind() == std::io::ErrorKind::NotFound => {
				dev_log!("lifecycle", "warn: [LandFix:VscodeFile] os-abs not on disk: {}", AbsolutePath);
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
	}

	dev_log!("lifecycle", "[LandFix:VscodeFile] Resolved path: {}", CleanPath);

	// Resolve against the frontendDist directory
	// In production: embedded in the binary via asset_resolver
	// In debug: fall back to filesystem read from Sky/Target
	let AssetResult = AppHandle.asset_resolver().get(CleanPath.clone());

	if let Some(Asset) = AssetResult {
		let Mime = MimeFromExtension::Fn(&CleanPath);

		let CacheControl = CacheControlFor(&CleanPath);

		let ETag = EmbeddedETag(&CleanPath, &Asset.bytes);

		if IfNoneMatchSatisfied(Request, &ETag) {
			dev_log!("scheme-assets", "[SchemeAssets] serve source=embedded path={} 304", CleanPath);

			return BuildNotModified(&ETag, CacheControl);
		}

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
			.header("ETag", ETag.as_str())
			.header("Cache-Control", CacheControl)
			.body(Asset.bytes.to_vec())
			.unwrap_or_else(|_| build_error_response(500, "Failed to build response"));
	}

	// Fallback: read from filesystem (dev mode where assets aren't embedded)
	let StaticRoot = crate::IPC::WindServiceHandlers::Utilities::ApplicationRoot::Get::Fn();

	if let Some(Root) = StaticRoot {
		let FilesystemPath = std::path::Path::new(&Root).join(&CleanPath);

		// LAND-PATCH B7.P01: mmap-cache the StaticRoot fallback
		// path so dev-mode workbench reloads pay the syscall
		// once per asset for the entire session. The open is the
		// existence probe - `NotFound` falls through to the 404
		// below without the former `exists()` / `is_file()`
		// double-stat.
		match ::Cache::AssetMemoryMap::LoadOrInsert::Fn(&FilesystemPath) {
			Ok(Entry) => {
				let CacheControl = CacheControlFor(&CleanPath);

				if IfNoneMatchSatisfied(Request, &Entry.ETag) {
					dev_log!("scheme-assets", "[LandFix:VscodeFile] fs-mmap 304 {}", CleanPath);

					return BuildNotModified(&Entry.ETag, CacheControl);
				}

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
					.header("ETag", Entry.ETag.as_str())
					.header("Cache-Control", CacheControl);

				if let Some(Enc) = Encoding {
					B = B.header("Content-Encoding", Enc);
				}

				return B
					.body(Body)
					.unwrap_or_else(|_| build_error_response(500, "Failed to build response"));
			},

			Err(Error) if Error.kind() == std::io::ErrorKind::NotFound => {},

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

	dev_log!(
		"lifecycle",
		"warn: [LandFix:VscodeFile] Not found: {} (resolved: {})",
		Uri,
		CleanPath
	);

	build_error_response(404, &format!("Not Found: {}", CleanPath))
}

/// Single-pass normalization of the post-authority resource path.
///
/// 1. Truncate at the first `?` or `#` so filesystem / asset-resolver
///    lookups operate on a clean path component. Roo's runtime
///    sourcemap-probe (`vZt` in its bundle) fetches
///    `<src>?source-map=true` which would otherwise hit the
///    asset_resolver as a literal `index.js?source-map=true` filename
///    and either 404 or fall through to the SPA-fallback `index.html`
///    (5765 bytes served as `application/octet-stream`). With the
///    strip, `index.js?source-map=true` → `index.js`, which exists on
///    disk and serves correctly with the right MIME. Sourcemap-probe
///    URLs that point to non-existent suffixes (`index.map.json`,
///    `index.sourcemap`) still 404 silently; that is the intended
///    behavior of `vZt`'s preload list.
/// 2. Strip the `/out/` prefix if present - our assets are at
///    `/Static/Application/vs/` not `/Static/Application/out/vs/`.
/// 3. Remap `Static/node_modules/` → `Static/Application/node_modules/`.
///    VS Code's nodeModulesPath = 'vs/../../node_modules' resolves
///    `../../` from `Static/Application/vs/` up to `Static/`; the
///    browser canonicalizes this to `Static/node_modules/` but our
///    files live at `Static/Application/node_modules/`.
///
/// Rules 2 and 3 are mutually exclusive on the prefix, so the single
/// `if`/`else` chain applies exactly the rewrite the former sequential
/// `replacen` passes did.
fn NormalizePath(FilePath:&str) -> String {
	let WithoutQuery = match FilePath.find(['?', '#']) {
		Some(Index) => &FilePath[..Index],

		None => FilePath,
	};

	if let Some(Rest) = WithoutQuery.strip_prefix("Static/Application//out/") {
		format!("Static/Application/{}", Rest)
	} else if let Some(Rest) = WithoutQuery.strip_prefix("Static/Application/out/") {
		format!("Static/Application/{}", Rest)
	} else if let Some(Rest) = WithoutQuery.strip_prefix("Static/node_modules/") {
		format!("Static/Application/node_modules/{}", Rest)
	} else {
		WithoutQuery.to_string()
	}
}

/// Cache-Control policy for a resolved path. Fingerprinted bundle
/// assets (content-hashed `js` / `css` / `woff2` filenames under the
/// `Static/` bundle tree) never change bytes under the same URL, so
/// they are safe to mark `immutable`. HTML and everything else -
/// including absolute-OS extension files, which can change on disk
/// between sessions - stays on a short max-age and revalidates via
/// `ETag` / `If-None-Match`.
fn CacheControlFor(CleanPath:&str) -> &'static str {
	if CleanPath.starts_with("Static/")
		&& (CleanPath.ends_with(".js") || CleanPath.ends_with(".css") || CleanPath.ends_with(".woff2"))
	{
		"public, max-age=31536000, immutable"
	} else {
		"public, max-age=3600"
	}
}

/// True when the request carries an `If-None-Match` header that
/// matches `ETag` (exact tag match or `*`).
fn IfNoneMatchSatisfied(Request:&tauri::http::request::Request<Vec<u8>>, ETag:&str) -> bool {
	Request
		.headers()
		.get("if-none-match")
		.and_then(|Value| Value.to_str().ok())
		.map(|Header| {
			Header.split(',').any(|Candidate| {
				let Trimmed = Candidate.trim();

				Trimmed == "*" || Trimmed == ETag
			})
		})
		.unwrap_or(false)
}

/// `304 Not Modified` carrying the same `ETag` / `Cache-Control` /
/// CORS surface as the corresponding `200` response, with an empty
/// body.
fn BuildNotModified(ETag:&str, CacheControl:&str) -> Response<Vec<u8>> {
	Builder::new()
		.status(304)
		.header("ETag", ETag)
		.header("Cache-Control", CacheControl)
		.header("Access-Control-Allow-Origin", "*")
		.header("Cross-Origin-Resource-Policy", "cross-origin")
		.body(Vec::new())
		.unwrap_or_else(|_| build_error_response(500, "Failed to build response"))
}

/// Weak ETag for embedded (asset_resolver) bytes. Embedded assets have
/// no mtime, so hash the content once per path and memoize the tag;
/// the format matches the mmap-cache `W/"<a>-<b>"` shape.
fn EmbeddedETag(CleanPath:&str, Bytes:&[u8]) -> String {
	use std::{
		collections::HashMap,
		hash::Hasher,
		sync::{OnceLock, RwLock},
	};

	static TAGS:OnceLock<RwLock<HashMap<String, String>>> = OnceLock::new();

	let Lock = TAGS.get_or_init(|| RwLock::new(HashMap::new()));

	if let Some(Existing) = Lock.read().ok().and_then(|Guard| Guard.get(CleanPath).cloned()) {
		return Existing;
	}

	let mut Hash = std::collections::hash_map::DefaultHasher::new();

	Hash.write(Bytes);

	let Tag = format!("W/\"{:x}-{:x}\"", Hash.finish(), Bytes.len());

	if let Ok(mut Guard) = Lock.write() {
		Guard.insert(CleanPath.to_string(), Tag.clone());
	}

	Tag
}
