#![allow(non_snake_case)]

//! NativeHost domain handlers for Wind IPC.
//!
//! Covers INativeHostService commands: dialogs, OS info, window state,
//! color scheme, dark mode detection, port scanning.

use std::{path::PathBuf, sync::Arc};

use serde_json::{Value, json};
use tauri::{AppHandle, Manager};

use crate::{
	ApplicationState::{
		ApplicationState,
		DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
		State::WorkspaceState::WorkspaceDelta::UpdateWorkspaceFoldersAndBroadcast,
	},
	dev_log,
};

/// Build a workspace folder DTO from the selected path so every pick-folder
/// flow goes through the same validated path → URI → name translation that
/// `MountainWorkspaceOpenFolder` uses. Returns `None` when the path is not a
/// directory or fails URL conversion.
fn BuildFolderDtoFromPath(Raw:&str, Index:usize) -> Option<WorkspaceFolderStateDTO> {
	let Path = PathBuf::from(Raw);
	if !Path.exists() {
		return None;
	}
	let Canonical = Path.canonicalize().unwrap_or(Path.clone());
	let Uri = url::Url::from_directory_path(&Canonical).ok()?;
	let Name = Canonical
		.file_name()
		.and_then(|N| N.to_str())
		.map(str::to_string)
		.unwrap_or_else(|| Canonical.display().to_string());
	WorkspaceFolderStateDTO::New(Uri, Name, Index).ok()
}

/// Pick folder using Tauri dialog plugin and reload webview with folder param.
///
/// After the dialog resolves but before navigating the webview, we push the
/// picked folder into `ApplicationState.Workspace` and fire the
/// `$deltaWorkspaceFolders` gRPC notification so Cocoon's extension host sees
/// the mutation before the new URL reloads the workbench. Without this step
/// the `workspaceContains:*` activation pass (BATCH-15) has nothing to match,
/// and `findFiles folders=0` persists because the Cocoon-side snapshot never
/// mirrored the pick.
pub async fn handle_native_pick_folder(AppHandle:AppHandle, _Args:Vec<Value>) -> Result<Value, String> {
	use tauri_plugin_dialog::DialogExt;

	dev_log!("folder", "pickFolderAndOpen requested");

	let Handle = AppHandle.clone();
	tokio::task::spawn_blocking(move || {
		let FolderPath = Handle.dialog().file().blocking_pick_folder();

		if let Some(Path) = FolderPath {
			let PathStr = Path.to_string();
			dev_log!("folder", "picked: {}", PathStr);

			// Mutate workspace state + emit $deltaWorkspaceFolders *before*
			// navigating - the webview reload discards the current Sky
			// context, but Cocoon keeps its state machine alive and must see
			// the delta to drive workspaceContains activation.
			if let Some(State) = Handle.try_state::<Arc<ApplicationState>>() {
				if let Some(Dto) = BuildFolderDtoFromPath(&PathStr, 0) {
					UpdateWorkspaceFoldersAndBroadcast(&Handle, &State.Workspace, vec![Dto]);
				} else {
					dev_log!(
						"folder",
						"warn: [pickFolderAndOpen] Failed to build WorkspaceFolderStateDTO for {}",
						PathStr
					);
				}
			} else {
				dev_log!(
					"folder",
					"warn: [pickFolderAndOpen] ApplicationState not managed by Tauri - workspace mutation skipped"
				);
			}

			if let Some(Window) = Handle.get_webview_window("main") {
				if let Ok(CurrentUrl) = Window.url() {
					let Origin = CurrentUrl.origin().unicode_serialization();
					let EncodedPath = url::form_urlencoded::Serializer::new(String::new())
						.append_pair("folder", &PathStr)
						.finish();
					let NewUrl = format!("{}/?{}", Origin, EncodedPath);
					dev_log!("folder", "navigating: {}", NewUrl);
					// Atom H1b: Cocoon liveness snapshot at the exact moment
					// of page reload. If Cocoon is alive here but the new
					// workbench never re-handshakes, the root cause is on
					// the Wind/Sky side; if dead, we need a re-spawn hook.
					dev_log!(
						"folder",
						"pre-nav cocoon-state: pid_known={} extension_host_ready_flag=see \"CocoonHealth healthy\" \
						 cadence below",
						Handle.try_state::<Arc<ApplicationState>>().is_some()
					);
					let _ = Window.navigate(NewUrl.parse().unwrap());
					dev_log!("folder", "post-nav Window.navigate() returned; webview reloading to {}", NewUrl);
				}
			}
		} else {
			dev_log!("folder", "pickFolderAndOpen cancelled by user");
		}
	});

	Ok(Value::Null)
}

/// Electron-style filter passed through `showOpenDialog({ filters: [...] })`.
/// Shape: `{ name: "VSIX Extensions", extensions: ["vsix"] }`. The tauri
/// dialog plugin's `add_filter(name, &[&str])` expects the same pair.
#[derive(Debug, Clone)]
struct DialogFilter {
	Name:String,
	Extensions:Vec<String>,
}

/// Parse `options.filters` into a vector of `DialogFilter`. Unknown / malformed
/// entries are silently skipped rather than failing the whole dialog open -
/// the user still gets the picker, just without the filter hint.
fn ParseDialogFilters(Options:&Value) -> Vec<DialogFilter> {
	Options
		.get("filters")
		.and_then(Value::as_array)
		.map(|Array| {
			Array
				.iter()
				.filter_map(|Entry| {
					let Name =
						Entry.get("name").and_then(Value::as_str).unwrap_or("Files").to_string();
					let Extensions:Vec<String> = Entry
						.get("extensions")
						.and_then(Value::as_array)
						.map(|List| {
							List.iter().filter_map(|V| V.as_str().map(str::to_string)).collect()
						})
						.unwrap_or_default();
					if Extensions.is_empty() { None } else { Some(DialogFilter { Name, Extensions }) }
				})
				.collect()
		})
		.unwrap_or_default()
}

/// Show open dialog with file/folder picker.
///
/// VS Code calls this via `nativeHostService.showOpenDialog(options)` and
/// expects the Electron contract:
///
///   - `properties: ["openDirectory" | "openFile" | "multiSelections" |
///      "createDirectory" | "showHiddenFiles"]`
///   - `filters: [{ name, extensions: ["vsix", …] }, …]`
///   - `title`, `buttonLabel`, `defaultPath`
///   - returns `{ canceled: bool, filePaths: string[] }`.
///
/// The VSIX install flow (`Install from VSIX…`) relies on `filters` to narrow
/// the picker to `.vsix` and on `openFile + multiSelections` so the user can
/// pick several archives at once. Without either, the dialog either never
/// opens (old stub) or opens unfiltered - both produced the "nothing happens"
/// symptom in the field. This handler drives the Tauri dialog plugin
/// end-to-end: every option in the VS Code contract maps to a builder call.
pub async fn handle_native_show_open_dialog(AppHandle:AppHandle, Args:Vec<Value>) -> Result<Value, String> {
	use tauri_plugin_dialog::DialogExt;

	dev_log!("folder", "showOpenDialog: {:?}", Args);

	// Electron passes `(windowId, options)`; `options` is always the last
	// element regardless of how the renderer was invoked. Searching by shape
	// (`first object with a "properties" or "filters" field`) keeps us robust
	// against VS Code versions that pass an extra prefix arg.
	let Options = Args
		.iter()
		.rev()
		.find(|V| V.is_object())
		.cloned()
		.unwrap_or(Value::Null);
	let Properties:Vec<String> = Options
		.get("properties")
		.and_then(Value::as_array)
		.map(|Array| Array.iter().filter_map(|V| V.as_str().map(str::to_string)).collect())
		.unwrap_or_default();
	let IsFolder = Properties.iter().any(|P| P == "openDirectory");
	let IsMultiple = Properties.iter().any(|P| P == "multiSelections");
	let Title = Options
		.get("title")
		.and_then(Value::as_str)
		.unwrap_or(if IsFolder { "Open Folder" } else { "Open File" })
		.to_string();
	let DefaultPath = Options.get("defaultPath").and_then(Value::as_str).map(str::to_string);
	// `filters` only affects file pickers; Tauri's folder picker ignores them.
	// Parsing unconditionally keeps the code branchless - the unused vector
	// costs nothing and we avoid an extra branch in the hot path.
	let Filters = ParseDialogFilters(&Options);

	let Handle = AppHandle.clone();
	let FiltersForThread = Filters.clone();
	let Selected = tokio::task::spawn_blocking(move || -> Vec<String> {
		let mut Builder = Handle.dialog().file().set_title(&Title);
		if let Some(Path) = DefaultPath.as_deref() {
			Builder = Builder.set_directory(Path);
		}
		// Apply filters only for file pickers - Tauri returns an error on
		// folder pickers if filters are set on some platforms.
		if !IsFolder {
			for Filter in &FiltersForThread {
				let ExtRefs:Vec<&str> = Filter.Extensions.iter().map(String::as_str).collect();
				Builder = Builder.add_filter(&Filter.Name, &ExtRefs);
			}
		}
		if IsFolder {
			if IsMultiple {
				Builder
					.blocking_pick_folders()
					.unwrap_or_default()
					.into_iter()
					.map(|P| P.to_string())
					.collect()
			} else {
				Builder.blocking_pick_folder().map(|P| vec![P.to_string()]).unwrap_or_default()
			}
		} else if IsMultiple {
			Builder
				.blocking_pick_files()
				.unwrap_or_default()
				.into_iter()
				.map(|P| P.to_string())
				.collect()
		} else {
			Builder.blocking_pick_file().map(|P| vec![P.to_string()]).unwrap_or_default()
		}
	})
	.await
	.map_err(|Error| format!("showOpenDialog join error: {}", Error))?;

	if Selected.is_empty() {
		dev_log!("folder", "showOpenDialog cancelled by user");
		Ok(json!({ "canceled": true, "filePaths": [] }))
	} else {
		dev_log!(
			"folder",
			"showOpenDialog selected {} path(s) (folder={}, multi={}, filters={})",
			Selected.len(),
			IsFolder,
			IsMultiple,
			Filters.len()
		);
		Ok(json!({ "canceled": false, "filePaths": Selected }))
	}
}

/// Get OS properties - cross-platform (macOS, Windows, Linux)
pub async fn handle_native_os_properties() -> Result<Value, String> {
	use sysinfo::System;

	let OsType = match std::env::consts::OS {
		"macos" => "Darwin",
		"windows" => "Windows_NT",
		"linux" => "Linux",
		_ => std::env::consts::OS,
	};

	let Release = {
		#[cfg(target_os = "macos")]
		{
			std::process::Command::new("sw_vers")
				.arg("-productVersion")
				.output()
				.ok()
				.map(|O| String::from_utf8_lossy(&O.stdout).trim().to_string())
				.unwrap_or_else(|| "14.0".to_string())
		}
		#[cfg(target_os = "windows")]
		{
			std::process::Command::new("cmd")
				.args(["/c", "ver"])
				.output()
				.ok()
				.map(|O| {
					let Output = String::from_utf8_lossy(&O.stdout);
					Output
						.split('[')
						.nth(1)
						.and_then(|S| S.split(']').next())
						.and_then(|S| S.strip_prefix("Version "))
						.unwrap_or("10.0.0")
						.to_string()
				})
				.unwrap_or_else(|| "10.0.0".to_string())
		}
		#[cfg(target_os = "linux")]
		{
			std::process::Command::new("uname")
				.arg("-r")
				.output()
				.ok()
				.map(|O| String::from_utf8_lossy(&O.stdout).trim().to_string())
				.unwrap_or_else(|| "6.1.0".to_string())
		}
		#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
		{
			"0.0.0".to_string()
		}
	};

	let mut Sys = System::new();
	Sys.refresh_cpu_all();
	let Cpus:Vec<Value> = Sys
		.cpus()
		.iter()
		.map(|Cpu| {
			json!({
				"model": Cpu.brand(),
				"speed": Cpu.frequency()
			})
		})
		.collect();

	Ok(json!({
		"type": OsType,
		"release": Release,
		"arch": std::env::consts::ARCH,
		"platform": std::env::consts::OS,
		"cpus": Cpus
	}))
}

/// Get OS statistics - cross-platform memory/load stats
pub async fn handle_native_os_statistics() -> Result<Value, String> {
	use sysinfo::System;

	let mut Sys = System::new();
	Sys.refresh_memory();

	let TotalMem = Sys.total_memory();
	let FreeMem = Sys.available_memory();

	let LoadAvg = {
		#[cfg(unix)]
		{
			let Load = System::load_average();
			vec![Load.one, Load.five, Load.fifteen]
		}
		#[cfg(not(unix))]
		{
			vec![0.0, 0.0, 0.0]
		}
	};

	Ok(json!({
		"totalmem": TotalMem,
		"freemem": FreeMem,
		"loadavg": LoadAvg
	}))
}

/// Check if window is fullscreen
pub async fn handle_native_is_fullscreen(AppHandle:AppHandle) -> Result<Value, String> {
	use tauri::Manager;
	let Window = AppHandle.get_webview_window("main");
	if let Some(W) = Window {
		Ok(json!(W.is_fullscreen().unwrap_or(false)))
	} else {
		Ok(json!(false))
	}
}

/// Check if window is maximized
pub async fn handle_native_is_maximized(AppHandle:AppHandle) -> Result<Value, String> {
	use tauri::Manager;
	let Window = AppHandle.get_webview_window("main");
	if let Some(W) = Window {
		Ok(json!(W.is_maximized().unwrap_or(false)))
	} else {
		Ok(json!(false))
	}
}

/// Find a free port starting from a given port
pub async fn handle_native_find_free_port(Args:Vec<Value>) -> Result<Value, String> {
	let StartPort = Args.get(0).and_then(|V| V.as_u64()).unwrap_or(9000) as u16;

	for Port in StartPort..StartPort + 100 {
		if std::net::TcpListener::bind(("127.0.0.1", Port)).is_ok() {
			return Ok(json!(Port));
		}
	}
	Ok(json!(0))
}

/// Detect OS color scheme - cross-platform dark mode detection
pub async fn handle_native_get_color_scheme() -> Result<Value, String> {
	let Dark = detect_dark_mode();
	let HighContrast = {
		#[cfg(target_os = "windows")]
		{
			std::process::Command::new("reg")
				.args(["query", "HKCU\\Control Panel\\Accessibility\\HighContrast", "/v", "Flags"])
				.output()
				.ok()
				.map(|O| {
					let Output = String::from_utf8_lossy(&O.stdout);
					Output.contains("0x1") || Output.contains("REG_DWORD    1")
				})
				.unwrap_or(false)
		}
		#[cfg(not(target_os = "windows"))]
		{
			#[cfg(target_os = "linux")]
			{
				std::process::Command::new("gsettings")
					.args(["get", "org.gnome.desktop.a11y.interface", "high-contrast"])
					.output()
					.ok()
					.map(|O| String::from_utf8_lossy(&O.stdout).trim() == "true")
					.unwrap_or(false)
			}
			#[cfg(not(target_os = "linux"))]
			{
				false
			}
		}
	};

	Ok(json!({ "dark": Dark, "highContrast": HighContrast }))
}

/// Cross-platform dark mode detection
fn detect_dark_mode() -> bool {
	#[cfg(target_os = "macos")]
	{
		std::process::Command::new("defaults")
			.args(["read", "-g", "AppleInterfaceStyle"])
			.output()
			.ok()
			.map(|O| String::from_utf8_lossy(&O.stdout).trim().to_lowercase().contains("dark"))
			.unwrap_or(false)
	}

	#[cfg(target_os = "windows")]
	{
		std::process::Command::new("reg")
			.args([
				"query",
				"HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
				"/v",
				"AppsUseLightTheme",
			])
			.output()
			.ok()
			.map(|O| {
				let Output = String::from_utf8_lossy(&O.stdout);
				Output.contains("0x0") || Output.contains("REG_DWORD    0")
			})
			.unwrap_or(false)
	}

	#[cfg(target_os = "linux")]
	{
		let GtkDark = std::process::Command::new("gsettings")
			.args(["get", "org.gnome.desktop.interface", "color-scheme"])
			.output()
			.ok()
			.map(|O| String::from_utf8_lossy(&O.stdout).contains("dark"))
			.unwrap_or(false);

		if GtkDark {
			return true;
		}

		let GtkTheme = std::process::Command::new("gsettings")
			.args(["get", "org.gnome.desktop.interface", "gtk-theme"])
			.output()
			.ok()
			.map(|O| String::from_utf8_lossy(&O.stdout).to_lowercase().contains("dark"))
			.unwrap_or(false);

		if GtkTheme {
			return true;
		}

		let KdeDark = std::env::var("KDE_COLOR_SCHEME")
			.ok()
			.map(|V| V.to_lowercase().contains("dark"))
			.unwrap_or(false);

		if KdeDark {
			return true;
		}

		let XfceDark = std::process::Command::new("xfconf-query")
			.args(["-c", "xsettings", "-p", "/Net/ThemeName"])
			.output()
			.ok()
			.map(|O| String::from_utf8_lossy(&O.stdout).to_lowercase().contains("dark"))
			.unwrap_or(false);

		XfceDark
	}

	#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
	{
		false
	}
}
