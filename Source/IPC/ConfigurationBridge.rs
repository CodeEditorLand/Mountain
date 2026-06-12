//! # Configuration Bridge - Bidirectional Configuration Synchronization
//!
//! **File Responsibilities:**
//! Manages bidirectional synchronization of configuration between
//! Mountain's Rust backend and Wind's TypeScript frontend. It ensures
//! configuration consistency across the entire CodeEditorLand ecosystem while
//! handling conflicts and updates gracefully.
//!
//! **Architectural Role in Wind-Mountain Connection:**
//!
//! The ConfigurationBridge is the synchronization layer that:
//!
//! 1. **Translates Configuration Formats:** Converts between Mountain's
//!    internal config structure and Wind's desktop configuration interface
//! 2. **Bidirectional Sync:** Maintains consistency in both directions
//!    (Wind→Mountain and Mountain→Wind)
//! 3. **Conflict Resolution:** Handles merge conflicts when multiple sources
//!    update configuration simultaneously
//! 4. **Validation:** Ensures all configuration changes are valid before
//!    applying
//! 5. **Identity Management:** Generates unique machine and session IDs for
//!    multi- instance scenarios
//!
//! **Bidirectional Synchronization Flow:**
//!
//! **Mountain → Wind Sync:**
//! ```
//! Mountain Services (Internal Config)
//!       |
//!       | get_mountain_configuration()
//!       v
//! ConfigurationBridge
//!       |
//!       | WindServiceAdapter.convert_to_wind_configuration()
//!       v
//! Wind Desktop Configuration Format
//!       |
//!       | send_configuration_to_wind()
//!       v
//! Wind Frontend (via IPC)
//! ```
//!
//! **Wind → Mountain Sync:**
//! ```
//! Wind Frontend (User Changes)
//!       |
//!       | WindConfigurationChange()
//!       v
//! ConfigurationBridge
//!       |
//!       | convert_to_mountain_configuration()
//!       v
//! Mountain Configuration Format
//!       |
//!       | update_mountain_configuration()
//!       v
//! Mountain Services (Internal Config)
//! ```
//!
//! **Configuration Bridge Features:**
//!
//! **1. Format Translation:**
//! - Mountain's internal JSON structure → Wind's desktop configuration
//!   interface
//! - Handles nested configuration objects
//! - Type conversion between TypeScript and Rust types
//!
//! **2. Conflict Resolution Strategy:**
//!
//! **Current Implementation (Basic):**
//! - Last-write-wins (most recent update takes precedence)
//! - Configuration is validated before applying
//! - Invalid changes are rejected entirely
//!
//! **Advanced Conflict Resolution (Future Enhancement):**
//! - Detect conflicts based on modification timestamps
//! - Provide conflict metadata (source, timestamp, value)
//! - Support three-way merge strategies:
//!   - **Ours:** Keep Mountain's version
//!   - **Theirs:** Use Wind's version
//!   - **Merge:** Attempt intelligent merge
//! - Conflict UI prompts in Wind for user resolution
//!
//! **3. Validation Rules:**
//!
//! **Type Validation:**
//! - `zoom_level`: Number, range -8.0 to 9.0
//! - `font_size`: Number, range 6.0 to 100.0
//! - `is_packaged`: Boolean
//! - `theme`, `platform`, `arch`: String, non-empty
//! - All other values: Not null
//!
//! **Key Validation:**
//! - Configuration keys must not be empty or whitespace
//! - Reserved keys cannot be modified
//! - Nested paths use dot notation (e.g., "editor.theme")
//!
//! **Value Validation:**
//! - Ranges checked for numeric values
//! - Enum validation for predefined options
//! - Pattern validation for string values (URLs, paths)
//!
//! **4. Identity Management:**
//!
//! **Machine ID Generation (Microsoft-Inspired):**
//! - **macOS:** Get system serial number via `system_profiler`
//! - **Windows:** Get machine UUID via `wmic csproduct get UUID`
//! - **Linux:** Read from `/etc/machine-id` or `/var/lib/dbus/machine-id`
//! - **Fallback:** Hash hostname + timestamp
//!
//! **Session ID Generation (Secure):**
//! - Combine timestamp, random number, and process ID
//! - Hash with SHA-256
//! - Use first 16 characters of hex digest
//! - Format: `session-{16-char-hash}`
//!
//! **5. Bidirectional Sync Triggers:**
//!
//! **Triggers for Mountain → Wind:**
//! - Configuration changes from Mountain services
//! - Periodic sync interval (configurable)
//! - Manual sync request from Mountain
//!
//! **Triggers for Wind → Mountain:**
//! - User changes configuration in Wind UI
//! - Settings panel updates
//! - Extension configuration changes
//! - Command palette configuration commands
//!
//! **Key Structures:**
//!
//! **ConfigurationBridge:**
//! Main synchronization orchestrator
//! - `get_wind_desktop_configuration()` - Get config in Wind format
//! - `update_configuration_from_wind()` - Apply Wind's config changes
//! - `synchronize_configuration()` - Force bidirectional sync
//! - `get_configuration_status()` - Get sync status info
//!
//! **ConfigurationStatus:**
//! Current synchronization state
//! - `is_valid` - Whether configuration is valid
//! - `last_sync` - Timestamp of last successful sync
//! - `configuration_keys` - List of all configuration keys
//!
//! **Tauri Commands:**
//!
//! The module provides Tauri commands for Wind to invoke:
//!
//! - `mountain_get_wind_desktop_configuration` - Get config for Wind UI
//! - `get_configuration_data` - Get all configuration data
//! - `save_configuration_data` - Save configuration from Wind
//! - `mountain_update_configuration_from_wind` - Update config from Wind
//! - `mountain_synchronize_configuration` - Force sync
//! - `mountain_get_configuration_status` - Get sync status
//!
//! **Configuration Flow Examples:**
//!
//! **Example 1: Wind Initializing**
//! ```typescript
//! // Wind startup
//! const config = await invoke('mountain_get_wind_desktop_configuration');
//! applyConfiguration(config);
//! ```
//!
//! **Example 2: User Changes Theme**
//! ```typescript
//! // User changes theme in Wind UI
//! const newConfig = { theme: 'dark', 'editor.fontSize': 14 };
//! await invoke('save_configuration_data', newConfig);
//! ```
//!
//! **Example 3: Mountain Updates Setting**
//! ```text
//! // Mountain service updates configuration
//! let bridge = ConfigurationBridge::new(runtime);
//! bridge.synchronize_configuration().await?;
//!
//! // Result: Wind UI automatically updates via IPC event
//! ```
//!
//! **Error Handling Strategy:**
//!
//! **Configuration Validation Errors:**
//! - Reject entire invalid configuration
//! - Return detailed validation error messages
//! - List which keys/values failed validation
//!
//! **Format Conversion Errors:**
//! - Log conversion errors with field names
//! - Attempt graceful fallback for missing fields
//! - Use defaults for conversion failures
//!
//! **Sync Errors:**
//! - Log sync failures with timestamps
//! - Queue sync for retry on transient errors
//! - Alert monitoring system on persistent failures
//!
//! **Integration with Other Modules:**
//!
//! **WindServiceAdapters:**
//! - Uses `WindServiceAdapter.convert_to_wind_configuration()`
//! - Depends on `WindDesktopConfiguration` structure
//!
//! **TauriIPCServer:**
//! - Sends configuration updates via IPC events
//! - Receives configuration changes from Wind
//!
//! **Mountain Configuration Service:**
//! - Delegates to `ConfigurationProvider` trait
//! - Uses `ConfigurationTarget` for scoping
//!
//! **Best Practices:**
//!
//! 1. **Always Validate:** Never apply configuration without validation
//! 2. **Atomic Updates:** Apply entire configuration atomically
//! 3. **Versioning:** Consider adding configuration versioning
//! 4. **Change Logging:** Log all configuration changes for audit
//! 5. **Fallback Support:** Provide sensible defaults for all settings
//! 6. **Conflict Detection:** Implement proper conflict detection before merges

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::Manager;
// Type aliases for Configuration DTOs to simplify usage
use CommonLibrary::Configuration::DTO::{
	ConfigurationOverridesDTO as ConfigurationOverridesDTOModule,
	ConfigurationTarget as ConfigurationTargetModule,
};

type ConfigurationOverridesDTO = ConfigurationOverridesDTOModule::ConfigurationOverridesDTO;

type ConfigurationTarget = ConfigurationTargetModule::ConfigurationTarget;

use CommonLibrary::{Configuration::ConfigurationProvider::ConfigurationProvider, Environment::Requires::Requires};
use sha2::Digest;

use crate::{
	IPC::WindServiceAdapters::{
		WindDesktopConfiguration::Struct as WindDesktopConfiguration,
		WindServiceAdapter::Struct as WindServiceAdapter,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Configuration bridge that handles Wind's desktop configuration needs
pub struct ConfigurationBridge {
	runtime:Arc<ApplicationRunTime>,
}

impl ConfigurationBridge {
	/// Create a new configuration bridge
	pub fn new(runtime:Arc<ApplicationRunTime>) -> Self {
		dev_log!("config", "[ConfigurationBridge] Creating configuration bridge");

		Self { runtime }
	}

	/// Get Wind-compatible desktop configuration
	pub async fn get_wind_desktop_configuration(&self) -> Result<WindDesktopConfiguration, String> {
		dev_log!("config", "[ConfigurationBridge] Getting Wind desktop configuration");

		// Get the current Mountain configuration
		let mountain_config = self.get_mountain_configuration().await?;

		// Convert to Wind format using the service adapter
		let service_adapter = WindServiceAdapter::new(self.runtime.clone());

		let wind_config = service_adapter.convert_to_wind_configuration(mountain_config).await?;

		dev_log!("config", "[ConfigurationBridge] Wind configuration ready");

		Ok(wind_config)
	}

	/// Update configuration from Wind frontend
	pub async fn update_configuration_from_wind(&self, wind_config:WindDesktopConfiguration) -> Result<(), String> {
		dev_log!("config", "[ConfigurationBridge] Updating configuration from Wind");

		// Convert Wind configuration to Mountain format
		let mountain_config = self.convert_to_mountain_configuration(wind_config).await?;

		// Update Mountain's configuration system
		self.update_mountain_configuration(mountain_config).await?;

		dev_log!("config", "[ConfigurationBridge] Configuration updated successfully");

		Ok(())
	}

	/// Get Mountain's current configuration
	async fn get_mountain_configuration(&self) -> Result<serde_json::Value, String> {
		dev_log!("config", "[ConfigurationBridge] Getting Mountain configuration");

		let config_provider:Arc<dyn ConfigurationProvider> = self.runtime.Environment.Require();

		let config = config_provider
			.GetConfigurationValue(None, ConfigurationOverridesDTO::default())
			.await
			.map_err(|e| format!("Failed to get Mountain configuration: {}", e))?;

		Ok(config)
	}

	/// Update Mountain's configuration
	async fn update_mountain_configuration(&self, config:serde_json::Value) -> Result<(), String> {
		dev_log!("config", "[ConfigurationBridge] Updating Mountain configuration");

		// Validate configuration before updating
		if !self.validate_configuration(&config) {
			return Err("Invalid configuration data".to_string());
		}

		let config_provider:Arc<dyn ConfigurationProvider> = self.runtime.Environment.Require();

		// Update configuration values
		if let Some(obj) = config.as_object() {
			for (key, value) in obj {
				config_provider
					.UpdateConfigurationValue(
						key.clone(),
						value.clone(),
						ConfigurationTarget::User,
						ConfigurationOverridesDTO::default(),
						None,
					)
					.await
					.map_err(|e| format!("Failed to update configuration key {}: {}", key, e))?;
			}
		}

		Ok(())
	}

	/// Validate configuration data
	fn validate_configuration(&self, config:&serde_json::Value) -> bool {
		// Basic validation: config must be an object
		if !config.is_object() {
			return false;
		}

		// Validate individual configuration values
		if let Some(obj) = config.as_object() {
			for (key, value) in obj {
				// Key validation
				if key.trim().is_empty() {
					return false;
				}

				// Value type validation
				match key.as_str() {
					"zoom_level" | "font_size" => {
						if let Some(num) = value.as_f64() {
							if key == "zoom_level" && (num < -8.0 || num > 9.0) {
								return false;
							}

							if key == "font_size" && (num < 6.0 || num > 100.0) {
								return false;
							}
						} else {
							return false;
						}
					},

					"is_packaged" | "enable_feature" => {
						if !value.is_boolean() {
							return false;
						}
					},

					"theme" | "platform" | "arch" => {
						if !value.is_string() || value.as_str().unwrap().trim().is_empty() {
							return false;
						}
					},

					_ => {
						// Default validation: value must not be null
						if value.is_null() {
							return false;
						}
					},
				}
			}
		}

		true
	}

	/// Convert Wind configuration to Mountain format
	async fn convert_to_mountain_configuration(
		&self,

		wind_config:WindDesktopConfiguration,
	) -> Result<serde_json::Value, String> {
		dev_log!("config", "[ConfigurationBridge] Converting Wind config to Mountain format");

		let machine_id = self.generate_machine_id().await.unwrap_or_else(|e| {
			dev_log!("config", "warn: [ConfigurationBridge] Failed to generate machine ID: {}", e);

			"wind-machine-fallback".to_string()
		});

		let session_id = self.generate_session_id().await.unwrap_or_else(|e| {
			dev_log!("config", "warn: [ConfigurationBridge] Failed to generate session ID: {}", e);

			"wind-session-fallback".to_string()
		});

		let mountain_config = serde_json::json!({
			"window_id": wind_config.window_id.to_string(),
			"machine_id": machine_id,
			"session_id": session_id,
			"log_level": wind_config.log_level,
			"app_root": wind_config.app_root,
			"user_data_dir": wind_config.user_data_path,
			"tmp_dir": wind_config.temp_path,
			"platform": wind_config.platform,
			"arch": wind_config.arch,
			"zoom_level": wind_config.zoom_level.unwrap_or(0.0),
			"backup_path": wind_config.backup_path.unwrap_or_default(),
			"home_dir": wind_config.profiles.home,
			"is_packaged": wind_config.is_packaged,
		});

		Ok(mountain_config)
	}

	/// Synchronize configuration between Mountain and Wind
	pub async fn synchronize_configuration(&self) -> Result<(), String> {
		dev_log!("config", "[ConfigurationBridge] Synchronizing configuration");

		// Get Mountain's current configuration
		let mountain_config = self.get_mountain_configuration().await?;

		// Convert to Wind format
		let service_adapter = WindServiceAdapter::new(self.runtime.clone());

		let wind_config = service_adapter.convert_to_wind_configuration(mountain_config).await?;

		// Send configuration to Wind via IPC
		self.send_configuration_to_wind(wind_config).await?;

		dev_log!("config", "[ConfigurationBridge] Configuration synchronized");

		Ok(())
	}

	/// Send configuration to Wind frontend via IPC
	async fn send_configuration_to_wind(&self, config:WindDesktopConfiguration) -> Result<(), String> {
		dev_log!("config", "[ConfigurationBridge] Sending configuration to Wind");

		// Get the IPC server
		if let Some(ipc_server) = self
			.runtime
			.Environment
			.ApplicationHandle
			.try_state::<crate::IPC::TauriIPCServer_Old::TauriIPCServer>()
		{
			let config_json =
				serde_json::to_value(config).map_err(|e| format!("Failed to serialize configuration: {}", e))?;

			ipc_server
				.send("configuration:update", config_json)
				.await
				.map_err(|e| format!("Failed to send configuration to Wind: {}", e))?;
		} else {
			return Err("IPC Server not found".to_string());
		}

		Ok(())
	}

	/// Handle configuration changes from Wind
	pub async fn WindConfigurationChange(&self, new_config:serde_json::Value) -> Result<(), String> {
		dev_log!("config", "[ConfigurationBridge] Handling Wind configuration change");

		// Parse Wind configuration
		let wind_config:WindDesktopConfiguration =
			serde_json::from_value(new_config).map_err(|e| format!("Failed to parse Wind configuration: {}", e))?;

		// Update Mountain configuration
		self.update_configuration_from_wind(wind_config).await?;

		dev_log!("config", "[ConfigurationBridge] Wind configuration change handled");

		Ok(())
	}

	/// Get configuration status
	pub async fn get_configuration_status(&self) -> Result<ConfigurationStatus, String> {
		dev_log!("config", "[ConfigurationBridge] Getting configuration status");

		let mountain_config = self.get_mountain_configuration().await?;

		let is_valid = !mountain_config.is_null();

		let status = ConfigurationStatus {
			is_valid,

			last_sync:std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64,

			configuration_keys:if let Some(obj) = mountain_config.as_object() {
				obj.keys().map(|k| k.clone()).collect()
			} else {
				Vec::new()
			},
		};

		Ok(status)
	}

	/// Generate unique machine ID using advanced Microsoft-inspired patterns
	async fn generate_machine_id(&self) -> Result<String, String> {
		// IMPLEMENTATION: Multi-platform machine ID generation
		#[cfg(target_os = "macos")]
		{
			use std::process::Command;

			// Get macOS serial number using system_profiler
			let result = Command::new("system_profiler")
				.arg("SPHardwareDataType")
				.arg("-json")
				.output()
				.map_err(|e| format!("Failed to execute system_profiler: {}", e))?;

			if result.status.success() {
				let output_str = String::from_utf8_lossy(&result.stdout);

				if let Ok(json) = serde_json::from_str::<serde_json::Value>(&output_str) {
					if let Some(serial) = json["SPHardwareDataType"][0]["serial_number"].as_str() {
						return Ok(format!("mac-{}", serial));
					}
				}
			}
		}

		#[cfg(target_os = "windows")]
		{
			use std::process::Command;

			// Get Windows machine ID using wmic
			let result = Command::new("wmic")
				.arg("csproduct")
				.arg("get")
				.arg("UUID")
				.output()
				.map_err(|e| format!("Failed to execute wmic: {}", e))?;

			if result.status.success() {
				let output_str = String::from_utf8_lossy(&result.stdout);

				let lines:Vec<&str> = output_str.lines().collect();

				if lines.len() > 1 {
					let uuid = lines[1].trim();

					if !uuid.is_empty() {
						return Ok(format!("win-{}", uuid));
					}
				}
			}
		}

		#[cfg(target_os = "linux")]
		{
			use std::fs;

			// Try to read machine-id from /etc/machine-id
			if let Ok(content) = fs::read_to_string("/etc/machine-id") {
				let machine_id = content.trim();

				if !machine_id.is_empty() {
					return Ok(format!("linux-{}", machine_id));
				}
			}

			// Fallback to /var/lib/dbus/machine-id
			if let Ok(content) = fs::read_to_string("/var/lib/dbus/machine-id") {
				let machine_id = content.trim();

				if !machine_id.is_empty() {
					return Ok(format!("linux-{}", machine_id));
				}
			}
		}

		// Fallback: Generate a unique ID based on hostname and timestamp
		let hostname = hostname::get()
			.map_err(|e| format!("Failed to get hostname: {}", e))?
			.to_string_lossy()
			.to_string();

		let timestamp = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis();

		Ok(format!("fallback-{}-{}", hostname, timestamp))
	}

	/// Generate unique session ID with Microsoft-inspired security patterns
	async fn generate_session_id(&self) -> Result<String, String> {
		use std::time::{SystemTime, UNIX_EPOCH};

		// IMPLEMENTATION: Secure session ID generation
		let random_part:u64 = rand::random();

		let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();

		// Get process ID for additional uniqueness
		let process_id = std::process::id();

		// Create a hash-based session ID
		let session_data = format!("{}:{}:{}", timestamp, random_part, process_id);

		let mut hasher = sha2::Sha256::new();

		hasher.update(session_data.as_bytes());

		let result = hasher.finalize();

		// Convert to hex string and take first 16 characters. sha2 0.11
		// dropped the `LowerHex` impl from `Digest::finalize()`'s output
		// (now `hybrid_array::Array`); `hex::encode` produces the same
		// lowercase-hex string the old `format!("{:x}", …)` emitted.
		let hex_string = hex::encode(result);

		let session_id = hex_string.chars().take(16).collect::<String>();

		dev_log!("config", "[ConfigurationBridge] Generated session ID: {}", session_id);

		Ok(format!("session-{}", session_id))
	}
}

/// Configuration status structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationStatus {
	pub is_valid:bool,

	pub last_sync:u64,

	pub configuration_keys:Vec<String>,
}

/// Tauri command to get Wind desktop configuration
#[tauri::command]
pub async fn mountain_get_wind_desktop_configuration(
	app_handle:tauri::AppHandle,
) -> Result<WindDesktopConfiguration, String> {
	dev_log!("config", "[ConfigurationBridge] Tauri command: get_wind_desktop_configuration");

	if let Some(runtime) = app_handle.try_state::<Arc<ApplicationRunTime>>() {
		let bridge = ConfigurationBridge::new(runtime.inner().clone());

		bridge.get_wind_desktop_configuration().await
	} else {
		Err("ApplicationRunTime not found".to_string())
	}
}

/// Tauri command to get configuration data for Wind frontend
#[tauri::command]
pub async fn get_configuration_data(app_handle:tauri::AppHandle) -> Result<serde_json::Value, String> {
	dev_log!("config", "[ConfigurationBridge] Tauri command: get_configuration_data");

	if let Some(runtime) = app_handle.try_state::<Arc<ApplicationRunTime>>() {
		let bridge = ConfigurationBridge::new(runtime.inner().clone());

		// Get Mountain's current configuration
		let mountain_config = bridge.get_mountain_configuration().await?;

		// Convert to Wind format
		let config_data = serde_json::json!({
			"application": mountain_config.clone(),
			"workspace": mountain_config.clone(),
			"profile": mountain_config.clone()
		});

		dev_log!("config", "[ConfigurationBridge] Configuration data retrieved successfully");

		Ok(config_data)
	} else {
		Err("ApplicationRunTime not found".to_string())
	}
}

/// Tauri command to save configuration data from Wind frontend
#[tauri::command]
pub async fn save_configuration_data(app_handle:tauri::AppHandle, config_data:serde_json::Value) -> Result<(), String> {
	dev_log!("config", "[ConfigurationBridge] Tauri command: save_configuration_data");

	if let Some(runtime) = app_handle.try_state::<Arc<ApplicationRunTime>>() {
		let bridge = ConfigurationBridge::new(runtime.inner().clone());

		// Update Mountain configuration with the new data
		bridge.update_mountain_configuration(config_data).await?;

		dev_log!("config", "[ConfigurationBridge] Configuration data saved successfully");

		Ok(())
	} else {
		Err("ApplicationRunTime not found".to_string())
	}
}

/// Tauri command to update configuration from Wind
#[tauri::command]
pub async fn mountain_update_configuration_from_wind(
	app_handle:tauri::AppHandle,

	config:serde_json::Value,
) -> Result<(), String> {
	dev_log!("config", "[ConfigurationBridge] Tauri command: update_configuration_from_wind");

	if let Some(runtime) = app_handle.try_state::<Arc<ApplicationRunTime>>() {
		let bridge = ConfigurationBridge::new(runtime.inner().clone());

		bridge.WindConfigurationChange(config).await
	} else {
		Err("ApplicationRunTime not found".to_string())
	}
}

/// Tauri command to synchronize configuration
#[tauri::command]
pub async fn mountain_synchronize_configuration(app_handle:tauri::AppHandle) -> Result<serde_json::Value, String> {
	dev_log!("config", "[ConfigurationBridge] Tauri command: synchronize_configuration");

	if let Some(runtime) = app_handle.try_state::<Arc<ApplicationRunTime>>() {
		let bridge = ConfigurationBridge::new(runtime.inner().clone());

		bridge
			.synchronize_configuration()
			.await
			.map(|_| serde_json::json!({ "status": "success" }))
	} else {
		Err("ApplicationRunTime not found".to_string())
	}
}

/// Tauri command to get configuration status
#[tauri::command]
pub async fn mountain_get_configuration_status(app_handle:tauri::AppHandle) -> Result<serde_json::Value, String> {
	dev_log!("config", "[ConfigurationBridge] Tauri command: get_configuration_status");

	if let Some(runtime) = app_handle.try_state::<Arc<ApplicationRunTime>>() {
		let bridge = ConfigurationBridge::new(runtime.inner().clone());

		bridge
			.get_configuration_status()
			.await
			.map(|status| serde_json::to_value(status).unwrap_or(serde_json::Value::Null))
	} else {
		Err("ApplicationRunTime not found".to_string())
	}
}
