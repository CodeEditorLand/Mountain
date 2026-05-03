//! # Window Build Module
//!
//! Creates and configures the main application window.

use tauri::{App, WebviewUrl, WebviewWindowBuilder, Wry};

use crate::IPC::WindServiceHandlers::Utilities::RecentlyOpened::ReadRecentlyOpened;

/// Creates and configures the main application window.
///
/// # Arguments
///
/// * `Application` - The Tauri application instance
/// * `LocalhostUrl` - The localhost URL for the webview content
///
/// # Returns
///
/// A configured `WebviewWindow<Wry>` instance.
///
/// # Platform-Specific Behavior
///
/// - **macOS**: `TitleBarStyle::Overlay` + `hidden_title(true)` keeps the
///   traffic-light buttons at the top-left but hides the native title bar
///   strip, so VS Code's custom titlebar (which has `-webkit-app-region: drag`
///   baked into its CSS) lights up the entire top row as a drag region. The
///   previous `maximized(true)` was the direct cause of "can't drag the editor
///   around" - maximized macOS windows are pinned to the screen and refuse all
///   drag events, regardless of `app-region` CSS.
/// - **Windows / Linux**: `decorations(false)` keeps the window chrome-less so
///   the workbench draws its own. We still start `resizable(true)` so the
///   window can be moved by the drag region.
/// - **Debug builds**: DevTools auto-open.
pub fn WindowBuild(Application:&mut App, LocalhostUrl:String) -> tauri::WebviewWindow<Wry> {
	// Restore the most-recently-opened folder so the webview boots
	// directly into the workspace. Without this, every launch lands
	// on the Welcome tab, the user clicks "Open Folder", and the
	// `pickFolderAndOpen` handler fires `Window.navigate()` - a hard
	// reload that wipes workbench state mid-initialisation and is
	// the direct cause of the purple splash flash, stuttering paint,
	// empty `@builtin` sidebar, and broken keybindings (every layer
	// has to boot twice and the second pass often loses references
	// to the first). Using `?folder=...` in the initial URL skips
	// that destructive round-trip.
	let InitialUrl = BuildInitialUrl(&LocalhostUrl);
	let WindowUrl = WebviewUrl::External(InitialUrl.parse().expect("FATAL: Failed to parse initial webview URL"));

	// Configure window builder with base settings.
	//
	// `visible(false)` is the hidden-until-ready pattern. Tauri's default
	// is to show the window the instant it's built, which paints the native
	// chrome + Base.astro's `#1e1e1e` inline background + VS Code theme CSS
	// + workbench DOM in four separate repaints over the first ~200 ms -
	// observed as the "purple/dark flash" and panel-pop flicker.
	//
	// Mountain shows the window explicitly when the frontend's
	// `lifecycle:advancePhase(3)` (Restored) arrives, which fires after
	// `.monaco-workbench` is attached and the first frame is ready. A 3 s
	// safety timer in `AppLifecycle` guarantees the window appears even if
	// Sky crashes before signalling phase 3.
	let mut WindowBuilder = WebviewWindowBuilder::new(Application, "main", WindowUrl)
		.use_https_scheme(false)
		.initialization_script("")
		.zoom_hotkeys_enabled(true)
		.browser_extensions_enabled(false)
		// macOS first-responder: by default WKWebView swallows the
		// first click on an unfocused window as a "make me key"
		// no-op and the click never reaches the inner content. With
		// `Inspect=1` running DevTools alongside the main window
		// every switch-back to the editor needed two clicks - first
		// to focus the NSWindow, second to actually focus the
		// Monaco textarea - which the user reports as "I clicked,
		// I'm typing, nothing's happening". `accept_first_mouse`
		// flips the responder chain so the first click already
		// reaches WKWebView's content and the textarea picks up
		// keyboard input immediately.
		.accept_first_mouse(true)
		.title("Mountain")
		.resizable(true)
		.inner_size(1400.0, 900.0)
		.shadow(true)
		.visible(false);

	#[cfg(target_os = "macos")]
	{
		// Overlay style lets VS Code's custom titlebar paint behind the
		// traffic-light buttons. `hidden_title(true)` suppresses the OS
		// title text so it doesn't collide with the workbench menubar.
		// `decorations(true)` is REQUIRED for the traffic lights to
		// render - turning decorations off also removes the buttons and
		// breaks the native drag + resize handles entirely on macOS.
		WindowBuilder = WindowBuilder
			.title_bar_style(tauri::TitleBarStyle::Overlay)
			.hidden_title(true)
			.decorations(true);
	}

	#[cfg(any(target_os = "windows", target_os = "linux"))]
	{
		// Keep chrome-less on non-macOS - the workbench provides its
		// own window controls via the WindowsTitleBar / LinuxTitleBar
		// components. Drag works via `-webkit-app-region: drag` on the
		// titlebar element.
		WindowBuilder = WindowBuilder.decorations(false);
	}

	// Build the main window
	let MainWindow = WindowBuilder.build().expect("FATAL: Main window build failed");

	// DevTools auto-open lives in `Binary/Main/AppLifecycle.rs:174`
	// (gated on `cfg(debug_assertions)`, with a `[Window] Debug build:
	// opening DevTools.` log line). Calling `open_devtools()` here as
	// well opened a SECOND DevTools window on every debug launch -
	// reported as "two DevTools" after the last rebuild. Single-source
	// the call to AppLifecycle so the log line and the window match.

	MainWindow
}

/// Build the initial webview URL, optionally appending `?folder=<path>`
/// when `~/.land/workspaces/RecentlyOpened.json` has an entry for the
/// previous session's workspace. Falls back to plain `index.html` if
/// the file is missing, malformed, or has no resolvable path.
///
/// The returned string is already URL-encoded and safe to feed to
/// `WebviewUrl::External`.
fn BuildInitialUrl(LocalhostUrl:&str) -> String {
	let Base = format!("{}/index.html", LocalhostUrl);

	let Recent = match ReadRecentlyOpened() {
		Ok(Value) => Value,
		Err(_) => return Base,
	};
	let Workspaces = match Recent.get("workspaces").and_then(|V| V.as_array()) {
		Some(Array) if !Array.is_empty() => Array,
		_ => return Base,
	};

	// VS Code's Recently-Opened record can store the folder under a few
	// different shapes depending on whether the entry came from the
	// extension host, the workbench, or a `$deltaWorkspaceFolders`
	// broadcast. Probe them in the same priority order the workbench
	// itself uses in `getRecentlyOpenedWorkspaces`.
	let Probe = |Entry:&serde_json::Value| -> Option<String> {
		// Mountain's own writer emits `{ uri: "file://…", label }` (see
		// `RecentlyOpened.json` on a freshly closed window). VS Code's
		// historical `folderUri` / `workspace.configPath` shapes are kept
		// as fallbacks so imported profiles and third-party writers keep
		// working.
		if let Some(Uri) = Entry.get("uri").and_then(|V| V.as_str()) {
			return Some(Uri.to_string());
		}
		if let Some(Uri) = Entry.get("folderUri").and_then(|V| V.as_str()) {
			return Some(Uri.to_string());
		}
		if let Some(Path) = Entry.get("folderUri").and_then(|V| V.get("path")).and_then(|V| V.as_str()) {
			return Some(Path.to_string());
		}
		if let Some(Path) = Entry
			.get("workspace")
			.and_then(|V| V.get("configPath"))
			.and_then(|V| V.get("path"))
			.and_then(|V| V.as_str())
		{
			return Some(Path.to_string());
		}
		None
	};

	let FolderPath = match Workspaces.iter().find_map(Probe) {
		Some(Path) => Path,
		None => return Base,
	};

	// Strip any `file://` scheme so the query param is a plain path
	// the workbench will stringify into a `file:` URI itself; leaving
	// the scheme in doubles up and breaks the URL-decode on the other
	// side (observed as the second `?folder=` boot path appearing as
	// `file:/Volumes/...` in `wb:boot`).
	let WithoutScheme = FolderPath.strip_prefix("file://").unwrap_or(FolderPath.as_str()).to_string();
	// RecentlyOpened.json stores workspace URIs with a trailing slash
	// (`file:///Volumes/.../Mountain/`). Drop it before encoding into
	// the URL so the workbench-side `URI.revive({ scheme: "file",
	// path: <param> })` produces a folder URI that matches the
	// workbench's own `URI.from(<file>)` results - which never carry
	// a trailing slash on the parent directory. The mismatch caused
	// `IUriIdentityService.extUri.relativePath` to return absolute
	// paths and breadcrumbs / quick-pick / Problems-panel labels to
	// render absolute `/Volumes/<vol>/...` paths instead of the workspace-relative
	// short form. Preserve `/` itself when the path IS root (vanishing
	// edge case but cheap to guard).
	let TrailingTrimmed = WithoutScheme.trim_end_matches('/');
	let Normalised = if TrailingTrimmed.is_empty() {
		"/".to_string()
	} else {
		TrailingTrimmed.to_string()
	};
	if !std::path::Path::new(&Normalised).is_dir() {
		return Base;
	}

	let Encoded = url::form_urlencoded::Serializer::new(String::new())
		.append_pair("folder", &Normalised)
		.finish();
	format!("{}/?{}", LocalhostUrl, Encoded)
}
