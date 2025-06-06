// ---------------------------------------------------------------------------------------------
// Mountain Sky Frontend Configuration Builder 
// --------------------------------------------------------------------------------------------
// This module is responsible for constructing the `SandboxConfigurationDto`,
// which provides essential initialization data to the Sky frontend. This DTO
// mirrors parts of VS Code's `ISandboxConfiguration` to facilitate
// compatibility and provide a familiar environment structure for frontend
// components.
//
// Responsibilities:
// - Gathering various pieces of information from the application environment:
//   - Platform, architecture, and OS details.
//   - Application paths (home, temp, user data, resources, executable).
//   - Application version and build information.
//   - NLS/localization settings.
//   - Product-specific details.
// - Populating the `SandboxConfigurationDto` with this data.
// - Providing helper functions for specific data points (e.g., webview
//   version).
//
// Key Interactions:
// - Called by `track.rs` when the `mountain_get_workbench_configuration` Tauri
//   command is invoked by Sky during its initialization.
// - Uses `tauri::AppHandle` to access `AppState`, `PathResolver`, and
//   `PackageInfo`.
// - Relies on `handlers::sky_dtos` for the DTO definitions.
// --------------------------------------------------------------------------------------------

use std::collections::HashMap;
use std::env; // For std::env::current_dir(), std::env::temp_dir(), std::env::consts
use std::path::PathBuf; // For path manipulation

use chrono::Utc; // For generating build date
use log::{debug, error, info, warn};
use tauri::utils::platform::current_exe; // To get executable path
use tauri::{AppHandle, Manager, Runtime, Wry}; // Wry is the default Tauri runtime
use uuid::Uuid;

use crate::app_state::AppState; // To get workspace ID for backup path
use crate::handlers::sky_dtos::{
	self,
	NlsConfigurationDto,
	ProcessVersionsDto,
	ProductConfigurationDto,
	SandboxConfigurationDto,
}; // For generating unique IDs (session, crash reporter)

/// Heuristic to get a string representing the webview version.
/// This is platform-dependent and might not be precise.
fn get_webview_version_heuristic() -> String {
	// In a real scenario, more robust detection or information passed from Sky
	// might be used.
	if cfg!(target_os = "windows") {
		"Edge WebView2/Unknown".to_string()
	} else if cfg!(target_os = "macos") {
		"WebKit/Unknown (macOS)".to_string()
	} else if cfg!(target_os = "linux") {
		"WebKitGTK/Unknown (Linux)".to_string()
	} else {
		"Unknown Webview".to_string()
	}
}

/// Maps `log::LevelFilter` to VS Code's numeric LogLevel enum values.
/// (Trace=0, Debug=1, Info=2, Warn=3, Error=4, Critical=5, Off=6)
fn map_log_level_filter_to_vscode_level(filter:log::LevelFilter) -> u32 {
	match filter {
		log::LevelFilter::Trace => 0,
		log::LevelFilter::Debug => 1,
		log::LevelFilter::Info => 2,
		log::LevelFilter::Warn => 3,
		log::LevelFilter::Error => 4,
		log::LevelFilter::Off => 6, // VS Code maps Critical to 5, Off to 6.
	}
}

/// Constructs the `SandboxConfigurationDto` to be sent to the Sky frontend.
pub fn build_sandbox_configuration(app_handle:&AppHandle<Wry>) -> SandboxConfigurationDto {
	info!("[Sky Config] Building ISandboxConfiguration for Sky frontend...");

	let app_state = app_handle.state::<AppState>();
	let path_resolver = app_handle.path_resolver();
	let package_info = app_handle.package_info();

	// --- Platform & Arch ---
	let platform_str = match env::consts::OS {
		"windows" => "win32",
		"macos" => "darwin",
		"linux" => "linux",
		_ => "unknown",
	}
	.to_string();
	let arch_str = match env::consts::ARCH {
		"x86_64" => "x64",
		"aarch64" => "arm64",
		"x86" => "ia32",
		_ => "unknown",
	}
	.to_string();

	// --- Paths ---
	let home_dir_path = path_resolver.home_dir().unwrap_or_else(|| {
		warn!("[Sky Config] Home directory not found. Using fallback '/fallback/home'.");
		PathBuf::from("/fallback/home")
	});
	let tmp_dir_path = env::temp_dir();
	let user_data_dir_path = path_resolver.app_data_dir().unwrap_or_else(|| {
		warn!("[Sky Config] App data directory not found. Using fallback '/fallback/appdata'.");
		PathBuf::from("/fallback/appdata")
	});
	let resources_dir_path = path_resolver.resource_dir().unwrap_or_else(|| {
		warn!("[Sky Config] Resource directory not found. Using fallback relative to CWD.");
		env::current_dir().unwrap_or_default().join("resources_fallback")
	});
	let exec_path_buf = current_exe().unwrap_or_else(|e| {
		warn!("[Sky Config] Failed to get current executable path: {}. Using fallback.", e);
		PathBuf::from("/fallback/executable_path")
	});

	// --- App Root URI ---
	// This needs to be carefully configured to match how Sky serves its assets.
	let app_root_uri = app_handle
		.config()
		.build
		.dev_url
		.as_ref()
		.map(|dev_url_val| {
			let base = dev_url_val.as_str();
			let base_with_slash = if base.ends_with('/') { base.to_string() } else { format!("{}/", base) };
			format!("{}Static/Application/", base_with_slash) // Assuming assets under Static/Application/
		})
		.unwrap_or_else(|| {
			// Default for production builds using Tauri's asset protocol.
			warn!(
				"[Sky Config] devUrl not found in tauri.conf.json. Falling back to 'app://-/Static/Application/' for \
				 appRoot. Ensure this is correct for your build."
			);
			"app://-/Static/Application/".to_string()
		});

	// --- NLS Configuration ---
	let mut available_langs = HashMap::new();
	available_langs.insert("en".to_string(), "English".to_string());
	// TODO: Populate `available_langs` from actual bundled NLS resources.
	// available_langs.insert("de".to_string(), "Deutsch".to_string());

	let current_language = env::var("LC_CTYPE")
		.or_else(|_| env::var("LANG"))
		.map(|loc_str| {
			loc_str
				.split('.')
				.next()
				.unwrap_or(&loc_str)
				.split('_')
				.next()
				.unwrap_or(&loc_str)
				.to_lowercase()
		})
		.unwrap_or_else(|_| "en".to_string()); // Default to 'en'

	// --- Versions DTO ---
	let versions_dto = ProcessVersionsDto {
		app_name:Some(package_info.name.clone()),
		app_version:Some(package_info.version.to_string()),
		tauri_version:Some(tauri::VERSION.to_string()), // Use tauri::VERSION for Tauri lib version
		webview_runtime_version:Some(get_webview_version_heuristic()),
	};

	// --- Product Configuration DTO ---
	let product_config_dto = ProductConfigurationDto {
		name_short:Some(package_info.name.chars().take(8).collect::<String>().to_uppercase()), // Example: "FIDDEE"
		name_long:Some(format!("{} Code Editor", package_info.name)),                          // Example
		application_name:Some(package_info.name.to_lowercase().replace(' ', "-")),             // e.g., "fiddee"
		version:Some(package_info.version.to_string()),
		commit:Some(env::var("SOURCE_COMMIT_HASH").unwrap_or_else(|_| "development".to_string())), /* Set via build
		                                                                                            * script */
		date:Some(Utc::now().to_rfc3339()), // Current build time or specific build date
		data_folder_name:Some(format!(".{}", package_info.name.to_lowercase())), // e.g., ".fiddee"
		embedder_identifier:Some("desktop".to_string()),
		additional_properties:HashMap::new(), // Populate if product.json has other fields
	};

	// --- Current Working Directory for VSCODE_CWD ---
	let vscode_cwd_path = env::current_dir()
		.map(|p| p.to_string_lossy().into_owned())
		.unwrap_or_else(|e| {
			warn!("[Sky Config] Failed to get CWD: {}. Using fallback '/fallback/cwd'.", e);
			"/fallback/cwd".to_string()
		});

	// --- User Environment Variables (Minimal Set) ---
	let user_env_vars = HashMap::new(); // Typically, don't pass all env vars.
	// Add specific ones if needed by extensions.

	// --- Backup Path ---
	// Workspace ID is needed for a unique backup path per workspace.
	let workspace_id_for_backup = app_state.get_workspace_id_string().unwrap_or_else(|err| {
		warn!(
			"[Sky Config] Failed to get workspace ID for backup path: {}. Using 'default_workspace'.",
			err
		);
		"default_workspace".to_string()
	});
	let backup_path_uri_string =
		sky_dtos::path_buf_to_uri_string(&user_data_dir_path.join("Backups").join(workspace_id_for_backup));

	// --- Construct the main SandboxConfigurationDto ---
	let config = SandboxConfigurationDto {
		window_id:app_handle.webview_window("main") // Assuming "main" is the label of the primary window
            .map(|w| w.label().to_string())
            .unwrap_or_else(|| {
                warn!("[Sky Config] Main window with label 'main' not found. Using fallback ID 'main_window_fallback'.");
                "main_window_fallback".to_string()
            }),
		machine_id:app_handle.manager().instance_id().to_string(), // Tauri's unique instance ID
		session_id:Uuid::new_v4().to_string(),                     // Generate a new session ID for each launch
		sqm_id:Some(app_handle.manager().instance_id().to_string()), // Can reuse machine_id or be specific
		log_level:map_log_level_filter_to_vscode_level(log::max_level()), // Current effective log level
		user_env:user_env_vars,
		app_root:app_root_uri,
		app_name:package_info.name.clone(),
		app_uri_scheme:package_info.name.to_lowercase(), // e.g., "fiddee", "vscode-resource"
		app_language:current_language.clone(),
		app_host:"desktop".to_string(), // Since this is a Tauri app
		product_quality:Some(if cfg!(debug_assertions) {
			"development".to_string()
		} else {
			"stable".to_string()
		}),
		platform:platform_str,
		arch:arch_str,
		versions:versions_dto,
		exec_path:exec_path_buf.to_string_lossy().into_owned(),
		zoom_level:Some(0.0), // Default zoom, Sky can manage this via commands
		home_dir:sky_dtos::path_buf_to_uri_string(&home_dir_path),
		tmp_dir:sky_dtos::path_buf_to_uri_string(&tmp_dir_path),
		user_data_dir:sky_dtos::path_buf_to_uri_string(&user_data_dir_path),
		backup_path:Some(backup_path_uri_string),
		crash_reporter_id:Some(Uuid::new_v4().to_string()), // Example crash reporter ID
		nls:NlsConfigurationDto {
			messages:HashMap::new(), // Assuming Sky loads messages.js
			language:current_language,
			available_languages:available_langs,
			pseudo:Some(false),
		},
		product_configuration:product_config_dto,
		vscode_cwd:Some(vscode_cwd_path),
		resources_path:resources_dir_path.to_string_lossy().into_owned(),
		additional_properties:HashMap::new(), // For any other dynamic fields VS Code might expect
	};

	info!("[Sky Config] ISandboxConfiguration DTO built successfully for Sky.");
	debug!(
		"[Sky Config] Config sample: AppRoot='{}', Platform='{}', UserDataDir URI='{}', WindowID='{}'",
		config.app_root, config.platform, config.user_data_dir, config.window_id
	);
	config
}
