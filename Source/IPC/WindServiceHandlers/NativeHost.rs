#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Native OS handlers - file picker, open external, OS info, window state,
//! port finding, color scheme detection.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{AppHandle, Manager};

use crate::{
	ApplicationState::{
		ApplicationState,
		DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
		State::WorkspaceState::WorkspaceDelta::UpdateWorkspaceFoldersAndBroadcast,
	},
	dev_log,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

/// Handler for showing items in folder
pub async fn handle_show_item_in_folder(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	let path_str = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	dev_log!("vfs", "showInFolder: {}", path_str);

	let path = std::path::PathBuf::from(path_str);

	// Validate path exists
	if !path.exists() {
		return Err(format!("Path does not exist: {}", path_str));
	}

	#[cfg(target_os = "macos")]
	{
		use std::process::Command;

		// Use macOS's open command with -R flag to reveal in Finder
		let result = Command::new("open")
			.arg("-R")
			.arg(&path)
			.output()
			.map_err(|e| format!("Failed to execute open command: {}", e))?;

		if !result.status.success() {
			return Err(format!(
				"Failed to show item in folder: {}",
				String::from_utf8_lossy(&result.stderr)
			));
		}
	}

	#[cfg(target_os = "windows")]
	{
		use std::process::Command;

		// Use Windows Explorer with /select flag
		let result = Command::new("explorer")
			.arg("/select,")
			.arg(&path)
			.output()
			.map_err(|e| format!("Failed to execute explorer command: {}", e))?;

		if !result.status.success() {
			return Err(format!(
				"Failed to show item in folder: {}",
				String::from_utf8_lossy(&result.stderr)
			));
		}
	}

	#[cfg(target_os = "linux")]
	{
		use std::process::Command;

		// Try common Linux file managers
		let file_managers = ["nautilus", "dolphin", "thunar", "pcmanfm", "nemo"];
		let mut last_error = String::new();

		for manager in file_managers.iter() {
			let result = Command::new(manager).arg(&path).output();

			match result {
				Ok(output) if output.status.success() => {
					dev_log!("lifecycle", "opened with {}", manager);
					break;
				},
				Err(e) => {
					last_error = e.to_string();
					continue;
				},
				_ => continue,
			}
		}

		if !last_error.is_empty() {
			return Err(format!("Failed to show item in folder with any file manager: {}", last_error));
		}
	}

	dev_log!("vfs", "showed in folder: {}", path_str);
	Ok(Value::Bool(true))
}

/// Handler for opening external URLs
pub async fn handle_open_external(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	let url_str = args
		.get(0)
		.ok_or("Missing URL".to_string())?
		.as_str()
		.ok_or("URL must be a string".to_string())?;

	dev_log!("lifecycle", "openExternal: {}", url_str);

	// Validate URL format
	if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
		return Err(format!("Invalid URL format. Must start with http:// or https://: {}", url_str));
	}

	#[cfg(target_os = "macos")]
	{
		use std::process::Command;

		let result = Command::new("open")
			.arg(url_str)
			.output()
			.map_err(|e| format!("Failed to execute open command: {}", e))?;

		if !result.status.success() {
			return Err(format!("Failed to open URL: {}", String::from_utf8_lossy(&result.stderr)));
		}
	}

	#[cfg(target_os = "windows")]
	{
		use std::process::Command;

		let result = Command::new("cmd")
			.arg("/c")
			.arg("start")
			.arg(url_str)
			.output()
			.map_err(|e| format!("Failed to execute start command: {}", e))?;

		if !result.status.success() {
			return Err(format!("Failed to open URL: {}", String::from_utf8_lossy(&result.stderr)));
		}
	}

	#[cfg(target_os = "linux")]
	{
		use std::process::Command;

		let handlers = ["xdg-open", "gnome-open", "kde-open", "x-www-browser"];
		let mut last_error = String::new();

		for handler in handlers.iter() {
			let result = Command::new(handler).arg(url_str).output();

			match result {
				Ok(output) if output.status.success() => {
					dev_log!("lifecycle", "opened with {}", handler);
					break;
				},
				Err(e) => {
					last_error = e.to_string();
					continue;
				},
				_ => continue,
			}
		}

		if !last_error.is_empty() {
			return Err(format!("Failed to open URL with any handler: {}", last_error));
		}
	}

	dev_log!("lifecycle", "opened URL: {}", url_str);
	Ok(Value::Bool(true))
}

/// Pick folder using Tauri dialog plugin and reload webview with folder param.
///
/// Atom I1 (2026-04-21): before the reload, mutate ApplicationState.Workspace
/// and fire `$deltaWorkspaceFolders` to Cocoon.
pub async fn handle_native_pick_folder(app_handle:AppHandle, _args:Vec<Value>) -> Result<Value, String> {
	use std::path::PathBuf;

	use tauri_plugin_dialog::DialogExt;

	dev_log!("folder", "pickFolderAndOpen requested");

	let Handle = app_handle.clone();
	tokio::task::spawn_blocking(move || {
		let FolderPath = Handle.dialog().file().blocking_pick_folder();

		if let Some(Path) = FolderPath {
			let PathStr = Path.to_string();
			dev_log!("folder", "picked: {}", PathStr);

			// Atom I1: synchronous workspace mutation + $deltaWorkspaceFolders broadcast.
			if let Some(State) = Handle.try_state::<Arc<ApplicationState>>() {
				let PathBuf = PathBuf::from(&PathStr);
				let Canonical = PathBuf.canonicalize().unwrap_or(PathBuf.clone());
				if let Ok(Uri) = url::Url::from_directory_path(&Canonical) {
					let Name = Canonical
						.file_name()
						.and_then(|N| N.to_str())
						.map(str::to_string)
						.unwrap_or_else(|| Canonical.display().to_string());
					match WorkspaceFolderStateDTO::New(Uri, Name, 0) {
						Ok(Dto) => {
							dev_log!("folder", "pre-nav workspace-delta: broadcasting 1 folder to Cocoon");
							UpdateWorkspaceFoldersAndBroadcast(&Handle, &State.Workspace, vec![Dto]);
						},
						Err(Error) => {
							dev_log!(
								"folder",
								"warn: [pickFolderAndOpen] WorkspaceFolderStateDTO::New failed: {}",
								Error
							);
						},
					}
				} else {
					dev_log!(
						"folder",
						"warn: [pickFolderAndOpen] path → file URI conversion failed for {}",
						PathStr
					);
				}
			} else {
				dev_log!(
					"folder",
					"warn: [pickFolderAndOpen] ApplicationState not managed by Tauri - delta skipped"
				);
			}

			// Navigate the webview to reload with the folder as workspace.
			if let Some(Window) = Handle.get_webview_window("main") {
				if let Ok(CurrentUrl) = Window.url() {
					let Origin = CurrentUrl.origin().unicode_serialization();
					let EncodedPath = url::form_urlencoded::Serializer::new(String::new())
						.append_pair("folder", &PathStr)
						.finish();
					let NewUrl = format!("{}/?{}", Origin, EncodedPath);
					dev_log!("folder", "navigating: {}", NewUrl);
					let _ = Window.navigate(NewUrl.parse().unwrap());
					dev_log!("folder", "post-nav Window.navigate() returned; webview reloading");
				}
			}
		} else {
			dev_log!("folder", "pickFolderAndOpen cancelled by user");
		}
	});

	Ok(Value::Null)
}

/// Show open dialog with file/folder picker
pub async fn handle_native_show_open_dialog(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	dev_log!("folder", "showOpenDialog: {:?}", args);
	// Return canceled for now - real dialog integration needs tauri_plugin_dialog
	Ok(json!({ "canceled": true, "filePaths": [] }))
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

	// Get OS release version
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

	// CPU info via sysinfo
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

	// Load average: available on Unix, not on Windows
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
pub async fn handle_native_is_fullscreen(app_handle:AppHandle) -> Result<Value, String> {
	let Window = app_handle.get_webview_window("main");
	if let Some(W) = Window {
		Ok(json!(W.is_fullscreen().unwrap_or(false)))
	} else {
		Ok(json!(false))
	}
}

/// Check if window is maximized
pub async fn handle_native_is_maximized(app_handle:AppHandle) -> Result<Value, String> {
	let Window = app_handle.get_webview_window("main");
	if let Some(W) = Window {
		Ok(json!(W.is_maximized().unwrap_or(false)))
	} else {
		Ok(json!(false))
	}
}

/// Find a free port starting from a given port
pub async fn handle_native_find_free_port(args:Vec<Value>) -> Result<Value, String> {
	let StartPort = args.get(0).and_then(|V| V.as_u64()).unwrap_or(9000) as u16;

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
	// High contrast detection
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
		// 1. GTK color-scheme (GNOME, Ubuntu, Fedora, etc.)
		let GtkDark = std::process::Command::new("gsettings")
			.args(["get", "org.gnome.desktop.interface", "color-scheme"])
			.output()
			.ok()
			.map(|O| String::from_utf8_lossy(&O.stdout).contains("dark"))
			.unwrap_or(false);

		if GtkDark {
			return true;
		}

		// 2. GTK theme name contains "dark"
		let GtkTheme = std::process::Command::new("gsettings")
			.args(["get", "org.gnome.desktop.interface", "gtk-theme"])
			.output()
			.ok()
			.map(|O| String::from_utf8_lossy(&O.stdout).to_lowercase().contains("dark"))
			.unwrap_or(false);

		if GtkTheme {
			return true;
		}

		// 3. KDE/Plasma
		let KdeDark = std::env::var("KDE_COLOR_SCHEME")
			.ok()
			.map(|V| V.to_lowercase().contains("dark"))
			.unwrap_or(false);

		if KdeDark {
			return true;
		}

		// 4. xfce4
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
