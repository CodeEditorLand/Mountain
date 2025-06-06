// ---------------------------------------------------------------------------------------------
// Mountain DTOs for Sky Frontend Initialization 
// --------------------------------------------------------------------------------------------
// This module defines Data Transfer Objects (DTOs) used to convey initial
// configuration and environment information from the Mountain backend to the
// Sky frontend. These structures often mirror parts of VS Code's
// `ISandboxConfiguration` and related interfaces to facilitate compatibility
// or reuse of VS Code workbench components in Sky.
// --------------------------------------------------------------------------------------------

use std::collections::HashMap;
// Using std::path::PathBuf as the source type for paths;
// tauri::PathBuf is often just a re-export or wrapper.
use std::path::PathBuf;

use serde::Serialize;
use url::Url; // For URI string fields

/// DTO for process and application version information.
/// Mirrors parts of `process` object and product versions in VS Code.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessVersionsDto {
	pub app_name:Option<String>,      // e.g., "fiddee"
	pub app_version:Option<String>,   // Application version (e.g., from Cargo.toml)
	pub tauri_version:Option<String>, // Tauri version used
	pub webview_runtime_version:Option<String>, /* e.g., "Chrome/1xx.x.x", "WebKit/xxx"
	                                   * Fields like node, electron, v8 versions are not applicable in a Tauri
	                                   * context. */
}

/// DTO for NLS (National Language Support) / localization configuration.
/// Mirrors `INLSConfiguration` from VS Code.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NlsConfigurationDto {
	/// Key-value pairs for localized strings. Typically empty if Sky loads
	/// `messages.js` or similar client-side.
	pub messages:HashMap<String, String>,
	/// Current UI language (e.g., "en", "de").
	pub language:String,
	/// Map of available languages and their display names (e.g., {"en":
	/// "English"}).
	pub available_languages:HashMap<String, String>,
	/// Whether pseudo-localization is enabled (for testing).
	pub pseudo:Option<bool>,
}

/// DTO for product-specific information (from `product.json` in VS Code).
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductConfigurationDto {
	pub name_short:Option<String>, // e.g., "FIDDEE"
	pub name_long:Option<String>,  // e.g., "FIDDEE Code Editor"
	/// The application name used in file associations, etc. (e.g., "fiddee" or
	/// "code").
	pub application_name:Option<String>,
	pub version:Option<String>, // Product version.
	pub commit:Option<String>,  // Git commit hash.
	pub date:Option<String>,    // Build date (ISO 8601 format).
	/// Name of the folder for user data (e.g., ".fiddee").
	pub data_folder_name:Option<String>,
	/// Identifier for the embedder environment (e.g., "desktop", "web").
	pub embedder_identifier:Option<String>,
	// Other fields from VS Code's product.json can be added here as needed:
	// e.g., quality, extensionsGallery, welcomePage, etc.
	#[serde(flatten)]
	pub additional_properties:HashMap<String, serde_json::Value>,
}

/// Main DTO for sandbox/workbench initialization data sent to Sky.
/// Mirrors VS Code's `ISandboxConfiguration`.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SandboxConfigurationDto {
	/// Unique identifier for the window. Can be a number or string.
	/// For Tauri, this might be the window label or a generated ID.
	pub window_id:String, // Changed to String for flexibility with Tauri labels
	/// Unique machine identifier.
	pub machine_id:String,
	/// Unique session identifier.
	pub session_id:String,
	/// ID for telemetry (Software Quality Metrics).
	pub sqm_id:Option<String>,
	/// Log level for the frontend (maps to VS Code LogLevel enum values).
	pub log_level:u32, // Trace=0, Debug=1, Info=2, Warn=3, Error=4, Critical=5, Off=6
	/// Environment variables relevant to the frontend or extensions.
	pub user_env:HashMap<String, Option<String>>,
	/// Base URI for application assets (e.g., "app://-/..." or "http://localhost:...").
	pub app_root:String,
	/// Application name.
	pub app_name:String,
	/// URI scheme used by the application (e.g., "app", "fiddee",
	/// "vscode-resource").
	pub app_uri_scheme:String,
	/// Current application language.
	pub app_language:String,
	/// Host environment ("desktop" or "web").
	pub app_host:String,
	/// Product quality ("stable", "insider", "development").
	pub product_quality:Option<String>,
	/// Platform identifier ("win32", "linux", "darwin").
	pub platform:String,
	/// Architecture ("x64", "arm64", "ia32").
	pub arch:String,
	/// Nested process/application versions.
	pub versions:ProcessVersionsDto,
	/// Filesystem path to the main Mountain executable.
	pub exec_path:String,
	/// Current zoom level of the window.
	pub zoom_level:Option<f64>,
	/// User's home directory as a file URI string.
	pub home_dir:String,
	/// Temporary directory as a file URI string.
	pub tmp_dir:String,
	/// User-specific application data directory as a file URI string.
	pub user_data_dir:String,
	/// Path for backups, as a file URI string.
	pub backup_path:Option<String>,
	/// Identifier for crash reporting.
	pub crash_reporter_id:Option<String>,
	/// NLS/localization configuration.
	pub nls:NlsConfigurationDto,
	/// Product-specific configuration.
	pub product_configuration:ProductConfigurationDto,
	/// Current working directory for VS Code compatibility (filesystem path).
	#[serde(rename = "VSCODE_CWD")]
	pub vscode_cwd:Option<String>,
	/// Path to application resources (filesystem path).
	pub resources_path:String,

	// Allow other dynamic fields for flexibility.
	#[serde(flatten)]
	pub additional_properties:HashMap<String, serde_json::Value>,
}

/// Helper function to convert a `std::path::PathBuf` to a file URI string.
pub fn path_buf_to_uri_string(path_buf:&PathBuf) -> String {
	Url::from_file_path(path_buf).map(|url| url.to_string()).unwrap_or_else(|_| {
		// Fallback for paths that might not convert cleanly.
		// Ensures forward slashes for URI compatibility.
		format!("file:///{}", path_buf.to_string_lossy().replace('\\', "/"))
	})
}
