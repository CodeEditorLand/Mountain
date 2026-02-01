//! # Wind Service Adapters - Type Conversion & Service Bridging
//!
//! **File Responsibilities:**
//! This module provides the adapter layer that handles type conversion and
//! service abstraction between Wind's TypeScript interfaces and Mountain's Rust
//! implementations. It allows Mountain services to present Wind-compatible APIs
//! while using Mountain's internal architecture.
//!
//! **Architectural Role in Wind-Mountain Connection:**
//!
//! The WindServiceAdapters module serves as the translation layer that:
//!
//! 1. **Converts Data Types:** Transforms TypeScript types to Rust types and
//!    vice versa
//! 2. **Abstracts Services:** Provides Wind-compatible service interfaces over
//!    Mountain services
//! 3. **Handles Configuration:** Converts between different configuration
//!    formats
//! 4. **Maintains Compatibility:** Ensures Wind contracts are satisfied
//!
//! **Adapter Pattern:**
//!
//! This module implements the Adapter design pattern to bridge the interface
//! gap:
//!
//! ```
//! Wind's IFileService (TypeScript interface)
//!        |
//!        |  Expected interface
//!        v
//! WindFileService (Rust adapter)
//!        |
//!        |  Delegates to
//!        v
//! Mountain's FileSystemReader (Rust trait)
//! ```
//!
//! **Key Structures:**
//!
//! **1. WindDesktopConfiguration:**
//! - Represents Wind's complete desktop configuration structure
//! - Mirrors Wind's TypeScript interface
//! - Includes window settings, paths, platform info, profile data
//!
//! **2. WindServiceAdapter:**
//! Main adapter that converts between Mountain and Wind formats
//! - `convert_to_wind_configuration()` - Mountain config to Wind config
//! - `get_environment_service()` - Wind-compatible environment service
//! - `get_file_service()` - Wind-compatible file service
//! - `get_storage_service()` - Wind-compatible storage service
//! - `get_configuration_service()` - Wind-compatible configuration service
//!
//! **3. Individual Service Adapters:**
//!
//! **WindEnvironmentService:**
//! Provides Wind-compatible environment variable access
//! - `get_app_root()` - Get application root path
//! - `get_user_data_path()` - Get user data directory
//!
//! **WindFileService:**
//! Adapts Mountain's file system to Wind's interface
//! - `read_file()` - Read file as bytes
//! - `write_file()` - Write file from bytes
//! - `stat_file()` - Get file metadata as JSON
//!
//! **WindStorageService:**
//! Adapts Mountain's storage to Wind's interface
//! - `get()` - Get storage value as JSON
//! - `set()` - Set storage value from JSON
//!
//! **WindConfigurationService:**
//! Adapts Mountain's configuration to Wind's interface
//! - `get_value()` - Get configuration value
//! - `update_value()` - Update configuration value
//!
//! **Type Conversion Examples:**
//!
//! **Configuration Conversion:**
//! ```typescript
//! // Wind TypeScript Configuration
//! interface IDesktopConfiguration {
//!   windowId: number;
//!   appRoot: string;
//!   userDataPath: string;
//!   // ... more fields
//! }
//! ```
//!
//! ```rust
//! // Mountain Rust Configuration (after conversion)
//! struct WindDesktopConfiguration {
//! 	pub window_id:u32,
//! 	pub app_root:String,
//! 	pub user_data_path:String,
//! 	// ... more fields
//! }
//! ```
//!
//! **File Service Integration:**
//!
//! Mountain's file system uses traits for abstraction:
//!
//! ```rust
//! let reader:Arc<dyn FileSystemReader> = runtime.Environment.Require();
//! let writer:Arc<dyn FileSystemWriter> = runtime.Environment.Require();
//!
//! // Adapt to Wind's interface
//! let wind_file_service = WindFileService::new(reader, writer);
//! let bytes = wind_file_service.read_file(path).await?;
//! ```
//!
//! **Configuration Bridge Integration:**
//!
//! The WindServiceAdapter works closely with ConfigurationBridge:
//! - ConfigurationBridge uses WindServiceAdapter to convert formats
//! - WindServiceAdapter maintains type compatibility
//! - Both work together to ensure seamless Wind-Mountain integration
//!
//! **Error Handling:**
//!
//! All adapter methods return `Result<T, String>` with descriptive errors:
//! - Type conversion errors include the field and reason
//! - Service delegation errors propagate with context
//! - All errors are in a format Wind can understand
//!
//! **Usage Pattern:**
//!
//! ```rust
//! // Create adapter
//! let adapter = WindServiceAdapter::new(runtime);
//!
//! // Get Mountain config
//! let mountain_config = get_mountain_config().await?;
//!
//! // Convert to Wind format
//! let wind_config = adapter.convert_to_wind_configuration(mountain_config).await?;
//!
//! // Get Wind-compatible services
//! let file_service = adapter.get_file_service().await?;
//! let config_service = adapter.get_configuration_service().await?;
//! ```

use std::{path::PathBuf, sync::Arc};

use log::{debug, info};
use serde::{Deserialize, Serialize};
use url::Url;
use CommonLibrary::{
	Configuration::{
		ConfigurationProvider::ConfigurationProvider,
		DTO::{
			ConfigurationOverridesDTO as ConfigurationOverridesDTOModule,
			ConfigurationTarget as ConfigurationTargetModule,
		},
	},
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	Storage::StorageProvider::StorageProvider,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
};

// Type aliases for Configuration DTOs to simplify usage
type ConfigurationOverridesDTO = ConfigurationOverridesDTOModule::ConfigurationOverridesDTO;
type ConfigurationTarget = ConfigurationTargetModule::ConfigurationTarget;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

/// Wind desktop configuration structure
/// Mirrors Wind's IDesktopConfiguration interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindDesktopConfiguration {
	pub window_id:u32,
	pub app_root:String,
	pub user_data_path:String,
	pub temp_path:String,
	pub log_level:String,
	pub is_packaged:bool,
	pub tauri_version:String,
	pub platform:String,
	pub arch:String,
	pub workspace:Option<serde_json::Value>,
	pub files_to_open_or_create:Option<Vec<FileToOpenOrCreate>>,
	pub files_to_diff:Option<Vec<FileToDiff>>,
	pub files_to_wait:Option<FilesToWait>,
	pub fullscreen:Option<bool>,
	pub zoom_level:Option<f64>,
	pub is_custom_zoom_level:Option<bool>,
	pub profiles:Profiles,
	pub policies_data:Option<serde_json::Value>,
	pub loggers:Vec<Logger>,
	pub backup_path:Option<String>,
	pub disable_layout_restore:Option<bool>,
	pub os:OsInfo,
}

/// File to open or create structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileToOpenOrCreate {
	pub file_uri:String,
}

/// File to diff structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileToDiff {
	pub file_uri:String,
}

/// Files to wait structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesToWait {
	pub wait_marker_file_uri:String,
	pub paths:Vec<FileToOpenOrCreate>,
}

/// Profiles structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profiles {
	pub all:Vec<serde_json::Value>,
	pub home:String,
	pub profile:serde_json::Value,
}

/// Logger structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logger {
	pub resource:serde_json::Value,
}

/// OS information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
	pub release:String,
}

/// Wind service adapter that bridges Mountain services to Wind's interfaces
pub struct WindServiceAdapter {
	runtime:Arc<ApplicationRunTime>,
}

impl WindServiceAdapter {
	/// Create a new Wind service adapter
	pub fn new(runtime:Arc<ApplicationRunTime>) -> Self {
		info!("[WindServiceAdapters] Creating Wind service adapter");
		Self { runtime }
	}

	/// Convert Mountain's sandbox configuration to Wind's desktop configuration
	pub async fn convert_to_wind_configuration(
		&self,
		mountain_config:serde_json::Value,
	) -> Result<WindDesktopConfiguration, String> {
		debug!("[WindServiceAdapters] Converting Mountain config to Wind config");

		// Parse the Mountain configuration
		let config:MountainSandboxConfiguration = serde_json::from_value(mountain_config)
			.map_err(|e| format!("Failed to parse Mountain configuration: {}", e))?;

		// Convert to Wind's format
		let wind_config = WindDesktopConfiguration {
			window_id:config.window_id.parse().unwrap_or(1),
			app_root:config.app_root,
			user_data_path:config.user_data_dir,
			temp_path:config.tmp_dir,
			log_level:config.log_level.to_string(),
			is_packaged:config.product_configuration.is_packaged,
			tauri_version:config.versions.mountain,
			platform:config.platform,
			arch:config.arch,
			workspace:None,
			files_to_open_or_create:None,
			files_to_diff:None,
			files_to_wait:None,
			fullscreen:Some(false),
			zoom_level:Some(config.zoom_level),
			is_custom_zoom_level:Some(false),
			profiles:Profiles { all:vec![], home:config.home_dir, profile:serde_json::Value::Null },
			policies_data:None,
			loggers:vec![],
			backup_path:Some(config.backup_path),
			disable_layout_restore:Some(false),
			os:OsInfo { release:std::env::consts::OS.to_string() },
		};

		Ok(wind_config)
	}

	/// Get Wind-compatible environment service
	pub async fn get_environment_service(&self) -> Result<WindEnvironmentService, String> {
		debug!("[WindServiceAdapters] Getting Wind environment service");

		Ok(WindEnvironmentService::new())
	}

	/// Get Wind-compatible file service
	pub async fn get_file_service(&self) -> Result<WindFileService, String> {
		debug!("[WindServiceAdapters] Getting Wind file service");

		let file_system_reader:Arc<dyn FileSystemReader> = self.runtime.Environment.Require();

		let file_system_writer:Arc<dyn FileSystemWriter> = self.runtime.Environment.Require();

		Ok(WindFileService::new(file_system_reader, file_system_writer))
	}

	/// Get Wind-compatible storage service
	pub async fn get_storage_service(&self) -> Result<WindStorageService, String> {
		debug!("[WindServiceAdapters] Getting Wind storage service");

		let storage:Arc<dyn StorageProvider> = self.runtime.Environment.Require();

		Ok(WindStorageService::new(storage))
	}

	/// Get Wind-compatible configuration service
	pub async fn get_configuration_service(&self) -> Result<WindConfigurationService, String> {
		debug!("[WindServiceAdapters] Getting Wind configuration service");

		let config:Arc<dyn ConfigurationProvider> = self.runtime.Environment.Require();

		Ok(WindConfigurationService::new(config))
	}
}

/// Wind environment service adapter
pub struct WindEnvironmentService {
	// Environment variables are accessed via std::env
}

impl WindEnvironmentService {
	pub fn new() -> Self { Self {} }

	pub async fn get_app_root(&self) -> Result<String, String> { std::env::var("APP_ROOT").map_err(|e| e.to_string()) }

	pub async fn get_user_data_path(&self) -> Result<String, String> {
		std::env::var("USER_DATA_PATH").map_err(|e| e.to_string())
	}
}

/// Wind file service adapter
pub struct WindFileService {
	reader:Arc<dyn FileSystemReader>,
	writer:Arc<dyn FileSystemWriter>,
}

impl WindFileService {
	pub fn new(reader:Arc<dyn FileSystemReader>, writer:Arc<dyn FileSystemWriter>) -> Self { Self { reader, writer } }

	pub async fn read_file(&self, path:String) -> Result<Vec<u8>, String> {
		self.reader.ReadFile(&PathBuf::from(path)).await.map_err(|e| e.to_string())
	}

	pub async fn write_file(&self, path:String, content:Vec<u8>) -> Result<(), String> {
		self.writer
			.WriteFile(&PathBuf::from(path), content, true, true)
			.await
			.map_err(|e:CommonError| e.to_string())
	}

	pub async fn stat_file(&self, path:String) -> Result<serde_json::Value, String> {
		self.reader
			.StatFile(&PathBuf::from(path))
			.await
			.map_err(|e:CommonError| e.to_string())
	}
}

/// Wind storage service adapter
pub struct WindStorageService {
	provider:Arc<dyn StorageProvider>,
}

impl WindStorageService {
	pub fn new(provider:Arc<dyn StorageProvider>) -> Self { Self { provider } }

	pub async fn get(&self, key:String) -> Result<serde_json::Value, String> {
		self.provider
			.GetStorageValue(false, &key)
			.await
			.map_err(|e:CommonError| e.to_string())
	}

	pub async fn set(&self, key:String, value:serde_json::Value) -> Result<(), String> {
		self.provider
			.UpdateStorageValue(false, key.to_string(), Some(value))
			.await
			.map_err(|e:CommonError| e.to_string())
	}
}

/// Wind configuration service adapter
pub struct WindConfigurationService {
	provider:Arc<dyn ConfigurationProvider>,
}

impl WindConfigurationService {
	pub fn new(provider:Arc<dyn ConfigurationProvider>) -> Self {
		Self { provider }
	}

	pub async fn get_value(&self, key:String) -> Result<serde_json::Value, String> {
		self.provider
			.GetConfigurationValue(Some(key.to_string()), ConfigurationOverridesDTO::default())
			.await
			.map_err(|e| e.to_string())
	}

	pub async fn update_value(&self, key:String, value:serde_json::Value) -> Result<(), String> {
		self.provider
			.UpdateConfigurationValue(
				key,
				value,
				ConfigurationTarget::User,
				ConfigurationOverridesDTO::default(),
				None,
			)
			.await
			.map_err(|e| e.to_string())
	}
}

/// Mountain sandbox configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MountainSandboxConfiguration {
	pub window_id:String,
	pub machine_id:String,
	pub session_id:String,
	pub log_level:i32,
	pub user_env:std::collections::HashMap<String, String>,
	pub app_root:String,
	pub app_name:String,
	pub app_uri_scheme:String,
	pub app_language:String,
	pub app_host:String,
	pub platform:String,
	pub arch:String,
	pub versions:Versions,
	pub exec_path:String,
	pub home_dir:String,
	pub tmp_dir:String,
	pub user_data_dir:String,
	pub backup_path:String,
	pub resources_path:String,
	pub vscode_cwd:String,
	pub nls:NLSConfiguration,
	pub product_configuration:ProductConfiguration,
	pub zoom_level:f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Versions {
	pub mountain:String,
	pub electron:String,
	pub chrome:String,
	pub node:String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NLSConfiguration {
	pub messages:std::collections::HashMap<String, String>,
	pub language:String,
	pub available_languages:std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProductConfiguration {
	pub name_short:String,
	pub name_long:String,
	pub application_name:String,
	pub embedder_identifier:String,
	pub is_packaged:bool,
}
