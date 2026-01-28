//! # Wind Service Adapters
//! 
//! Mountain adapters that bridge Wind's TypeScript service interfaces to Mountain's Rust services.
//! Provides seamless integration between Wind's desktop services and Mountain's backend services.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

/// Wind desktop configuration structure
/// Mirrors Wind's IDesktopConfiguration interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindDesktopConfiguration {
    pub window_id: u32,
    pub app_root: String,
    pub user_data_path: String,
    pub temp_path: String,
    pub log_level: String,
    pub is_packaged: bool,
    pub tauri_version: String,
    pub platform: String,
    pub arch: String,
    pub workspace: Option<serde_json::Value>,
    pub files_to_open_or_create: Option<Vec<FileToOpenOrCreate>>,
    pub files_to_diff: Option<Vec<FileToDiff>>,
    pub files_to_wait: Option<FilesToWait>,
    pub fullscreen: Option<bool>,
    pub zoom_level: Option<f64>,
    pub is_custom_zoom_level: Option<bool>,
    pub profiles: Profiles,
    pub policies_data: Option<serde_json::Value>,
    pub loggers: Vec<Logger>,
    pub backup_path: Option<String>,
    pub disable_layout_restore: Option<bool>,
    pub os: OsInfo,
}

/// File to open or create structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileToOpenOrCreate {
    pub file_uri: String,
}

/// File to diff structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileToDiff {
    pub file_uri: String,
}

/// Files to wait structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesToWait {
    pub wait_marker_file_uri: String,
    pub paths: Vec<FileToOpenOrCreate>,
}

/// Profiles structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profiles {
    pub all: Vec<serde_json::Value>,
    pub home: String,
    pub profile: serde_json::Value,
}

/// Logger structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logger {
    pub resource: serde_json::Value,
}

/// OS information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub release: String,
}

/// Wind service adapter that bridges Mountain services to Wind's interfaces
pub struct WindServiceAdapter {
    runtime: Arc<ApplicationRunTime>,
}

impl WindServiceAdapter {
    /// Create a new Wind service adapter
    pub fn new(runtime: Arc<ApplicationRunTime>) -> Self {
        info!("[WindServiceAdapters] Creating Wind service adapter");
        Self { runtime }
    }

    /// Convert Mountain's sandbox configuration to Wind's desktop configuration
    pub async fn convert_to_wind_configuration(
        &self,
        mountain_config: serde_json::Value,
    ) -> Result<WindDesktopConfiguration, String> {
        debug!("[WindServiceAdapters] Converting Mountain config to Wind config");
        
        // Parse the Mountain configuration
        let config: MountainSandboxConfiguration = serde_json::from_value(mountain_config)
            .map_err(|e| format!("Failed to parse Mountain configuration: {}", e))?;
        
        // Convert to Wind's format
        let wind_config = WindDesktopConfiguration {
            window_id: config.window_id.parse().unwrap_or(1),
            app_root: config.app_root,
            user_data_path: config.user_data_dir,
            temp_path: config.tmp_dir,
            log_level: config.log_level.to_string(),
            is_packaged: config.product_configuration.is_packaged,
            tauri_version: config.versions.mountain,
            platform: config.platform,
            arch: config.arch,
            workspace: None,
            files_to_open_or_create: None,
            files_to_diff: None,
            files_to_wait: None,
            fullscreen: Some(false),
            zoom_level: Some(config.zoom_level),
            is_custom_zoom_level: Some(false),
            profiles: Profiles {
                all: vec![],
                home: config.home_dir,
                profile: serde_json::Value::Null,
            },
            policies_data: None,
            loggers: vec![],
            backup_path: Some(config.backup_path),
            disable_layout_restore: Some(false),
            os: OsInfo {
                release: std::env::consts::OS.to_string(),
            },
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
        
        let file_system: Arc<dyn Common::FileSystem::FileSystemReader> = 
            self.runtime.Environment.Require();
        
        Ok(WindFileService::new(file_system))
    }

    /// Get Wind-compatible storage service
    pub async fn get_storage_service(&self) -> Result<WindStorageService, String> {
        debug!("[WindServiceAdapters] Getting Wind storage service");
        
        let storage: Arc<dyn Common::Storage::StorageProvider::StorageProvider> = 
            self.runtime.Environment.Require();
        
        Ok(WindStorageService::new(storage))
    }

    /// Get Wind-compatible configuration service
    pub async fn get_configuration_service(&self) -> Result<WindConfigurationService, String> {
        debug!("[WindServiceAdapters] Getting Wind configuration service");
        
        let config: Arc<dyn Common::Configuration::ConfigurationProvider::ConfigurationProvider> = 
            self.runtime.Environment.Require();
        
        Ok(WindConfigurationService::new(config))
    }
}

/// Wind environment service adapter
pub struct WindEnvironmentService {
    // Environment variables are accessed via std::env
}

impl WindEnvironmentService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn get_app_root(&self) -> Result<String, String> {
        std::env::var("APP_ROOT")
            .map_err(|e| e.to_string())
    }

    pub async fn get_user_data_path(&self) -> Result<String, String> {
        std::env::var("USER_DATA_PATH")
            .map_err(|e| e.to_string())
    }
}

/// Wind file service adapter
pub struct WindFileService {
    provider: Arc<dyn Common::FileSystem::FileSystemReader>,
}

impl WindFileService {
    pub fn new(provider: Arc<dyn Common::FileSystem::FileSystemReader>) -> Self {
        Self { provider }
    }

    pub async fn read_file(&self, path: String) -> Result<Vec<u8>, String> {
        self.provider.ReadFile(path)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn write_file(&self, path: String, content: Vec<u8>) -> Result<(), String> {
        self.provider.WriteFile(path, content, true, true)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn stat_file(&self, path: String) -> Result<serde_json::Value, String> {
        self.provider.StatFile(path)
            .await
            .map_err(|e| e.to_string())
    }
}

/// Wind storage service adapter
pub struct WindStorageService {
    provider: Arc<dyn Common::Storage::StorageProvider::StorageProvider>,
}

impl WindStorageService {
    pub fn new(provider: Arc<dyn Common::Storage::StorageProvider::StorageProvider>) -> Self {
        Self { provider }
    }

    pub async fn get(&self, key: String) -> Result<serde_json::Value, String> {
        self.provider.GetStorageItem(key)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn set(&self, key: String, value: serde_json::Value) -> Result<(), String> {
        self.provider.SetStorageItem(key, value)
            .await
            .map_err(|e| e.to_string())
    }
}

/// Wind configuration service adapter
pub struct WindConfigurationService {
    provider: Arc<dyn Common::Configuration::ConfigurationProvider::ConfigurationProvider>,
}

impl WindConfigurationService {
    pub fn new(provider: Arc<dyn Common::Configuration::ConfigurationProvider::ConfigurationProvider>) -> Self {
        Self { provider }
    }

    pub async fn get_value(&self, key: String) -> Result<serde_json::Value, String> {
        self.provider.GetConfigurationValue(key, serde_json::Value::Null)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn update_value(&self, key: String, value: serde_json::Value) -> Result<(), String> {
        self.provider.UpdateConfigurationValue(
            key,
            value,
            Common::Configuration::DTO::ConfigurationTarget::ConfigurationTarget::User,
            serde_json::Value::Null,
            None,
        )
        .await
        .map_err(|e| e.to_string())
    }
}

/// Mountain sandbox configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MountainSandboxConfiguration {
    pub window_id: String,
    pub machine_id: String,
    pub session_id: String,
    pub log_level: i32,
    pub user_env: std::collections::HashMap<String, String>,
    pub app_root: String,
    pub app_name: String,
    pub app_uri_scheme: String,
    pub app_language: String,
    pub app_host: String,
    pub platform: String,
    pub arch: String,
    pub versions: Versions,
    pub exec_path: String,
    pub home_dir: String,
    pub tmp_dir: String,
    pub user_data_dir: String,
    pub backup_path: String,
    pub resources_path: String,
    pub vscode_cwd: String,
    pub nls: NLSConfiguration,
    pub product_configuration: ProductConfiguration,
    pub zoom_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Versions {
    pub mountain: String,
    pub electron: String,
    pub chrome: String,
    pub node: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NLSConfiguration {
    pub messages: std::collections::HashMap<String, String>,
    pub language: String,
    pub available_languages: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProductConfiguration {
    pub name_short: String,
    pub name_long: String,
    pub application_name: String,
    pub embedder_identifier: String,
    pub is_packaged: bool,
}
