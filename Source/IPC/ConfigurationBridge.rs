//! # Configuration Bridge
//! 
//! Bridges Mountain's configuration system to Wind's desktop configuration requirements.
//! Ensures seamless configuration sharing between Mountain backend and Wind frontend.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};

use crate::{
    IPC::WindServiceAdapters::{WindDesktopConfiguration, WindServiceAdapter},
    RunTime::ApplicationRunTime::ApplicationRunTime,
};

/// Configuration bridge that handles Wind's desktop configuration needs
pub struct ConfigurationBridge {
    runtime: Arc<ApplicationRunTime>,
}

impl ConfigurationBridge {
    /// Create a new configuration bridge
    pub fn new(runtime: Arc<ApplicationRunTime>) -> Self {
        info!("[ConfigurationBridge] Creating configuration bridge");
        Self { runtime }
    }

    /// Get Wind-compatible desktop configuration
    pub async fn get_wind_desktop_configuration(&self) -> Result<WindDesktopConfiguration, String> {
        debug!("[ConfigurationBridge] Getting Wind desktop configuration");
        
        // Get the current Mountain configuration
        let mountain_config = self.get_mountain_configuration().await?;
        
        // Convert to Wind format using the service adapter
        let service_adapter = WindServiceAdapter::new(self.runtime.clone());
        let wind_config = service_adapter.convert_to_wind_configuration(mountain_config).await?;
        
        debug!("[ConfigurationBridge] Wind configuration ready");
        Ok(wind_config)
    }

    /// Update configuration from Wind frontend
    pub async fn update_configuration_from_wind(
        &self,
        wind_config: WindDesktopConfiguration,
    ) -> Result<(), String> {
        debug!("[ConfigurationBridge] Updating configuration from Wind");
        
        // Convert Wind configuration to Mountain format
        let mountain_config = self.convert_to_mountain_configuration(wind_config).await?;
        
        // Update Mountain's configuration system
        self.update_mountain_configuration(mountain_config).await?;
        
        debug!("[ConfigurationBridge] Configuration updated successfully");
        Ok(())
    }

    /// Get Mountain's current configuration
    async fn get_mountain_configuration(&self) -> Result<serde_json::Value, String> {
        debug!("[ConfigurationBridge] Getting Mountain configuration");
        
        let config_provider: Arc<dyn Common::Configuration::ConfigurationProvider::ConfigurationProvider> = 
            self.runtime.Environment.Require();
        
        let config = config_provider.GetConfiguration(None, serde_json::Value::Null)
            .await
            .map_err(|e| format!("Failed to get Mountain configuration: {}", e))?;
        
        Ok(config)
    }

    /// Update Mountain's configuration
    async fn update_mountain_configuration(&self, config: serde_json::Value) -> Result<(), String> {
        debug!("[ConfigurationBridge] Updating Mountain configuration");
        
        let config_provider: Arc<dyn Common::Configuration::ConfigurationProvider::ConfigurationProvider> = 
            self.runtime.Environment.Require();
        
        // Update configuration values
        if let Some(obj) = config.as_object() {
            for (key, value) in obj {
                config_provider.UpdateConfigurationValue(
                    key.clone(),
                    value.clone(),
                    Common::Configuration::DTO::ConfigurationTarget::ConfigurationTarget::User,
                    serde_json::Value::Null,
                    None,
                )
                .await
                .map_err(|e| format!("Failed to update configuration key {}: {}", key, e))?;
            }
        }
        
        Ok(())
    }

    /// Convert Wind configuration to Mountain format
    async fn convert_to_mountain_configuration(
        &self,
        wind_config: WindDesktopConfiguration,
    ) -> Result<serde_json::Value, String> {
        debug!("[ConfigurationBridge] Converting Wind config to Mountain format");
        
        // ADVANCED IMPLEMENTATION: Microsoft-inspired machine and session ID generation
        let machine_id = self.generate_machine_id().await
            .unwrap_or_else(|e| {
                warn!("[ConfigurationBridge] Failed to generate machine ID: {}", e);
                "wind-machine-fallback".to_string()
            });
        
        let session_id = self.generate_session_id().await
            .unwrap_or_else(|e| {
                warn!("[ConfigurationBridge] Failed to generate session ID: {}", e);
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
        debug!("[ConfigurationBridge] Synchronizing configuration");
        
        // Get Mountain's current configuration
        let mountain_config = self.get_mountain_configuration().await?;
        
        // Convert to Wind format
        let service_adapter = WindServiceAdapter::new(self.runtime.clone());
        let wind_config = service_adapter.convert_to_wind_configuration(mountain_config).await?;
        
        // Send configuration to Wind via IPC
        self.send_configuration_to_wind(wind_config).await?;
        
        debug!("[ConfigurationBridge] Configuration synchronized");
        Ok(())
    }

    /// Send configuration to Wind frontend via IPC
    async fn send_configuration_to_wind(&self, config: WindDesktopConfiguration) -> Result<(), String> {
        debug!("[ConfigurationBridge] Sending configuration to Wind");
        
        // Get the IPC server
        if let Some(ipc_server) = self.runtime.Environment.ApplicationHandle.try_state::<crate::IPC::TauriIPCServer::TauriIPCServer>() {
            let config_json = serde_json::to_value(config)
                .map_err(|e| format!("Failed to serialize configuration: {}", e))?;
            
            ipc_server.send("configuration:update", config_json).await
                .map_err(|e| format!("Failed to send configuration to Wind: {}", e))?;
        } else {
            return Err("IPC Server not found".to_string());
        }
        
        Ok(())
    }

    /// Handle configuration changes from Wind
    pub async fn handle_wind_configuration_change(
        &self,
        new_config: serde_json::Value,
    ) -> Result<(), String> {
        debug!("[ConfigurationBridge] Handling Wind configuration change");
        
        // Parse Wind configuration
        let wind_config: WindDesktopConfiguration = serde_json::from_value(new_config)
            .map_err(|e| format!("Failed to parse Wind configuration: {}", e))?;
        
        // Update Mountain configuration
        self.update_configuration_from_wind(wind_config).await?;
        
        debug!("[ConfigurationBridge] Wind configuration change handled");
        Ok(())
    }

    /// Get configuration status
    pub async fn get_configuration_status(&self) -> Result<ConfigurationStatus, String> {
        debug!("[ConfigurationBridge] Getting configuration status");
        
        let mountain_config = self.get_mountain_configuration().await?;
        let is_valid = !mountain_config.is_null();
        
        let status = ConfigurationStatus {
            is_valid,
            last_sync: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            configuration_keys: if let Some(obj) = mountain_config.as_object() {
                obj.keys().map(|k| k.clone()).collect()
            } else {
                Vec::new()
            },
        };
        
        Ok(status)
    }

    /// Generate unique machine ID using advanced Microsoft-inspired patterns
    async fn generate_machine_id(&self) -> Result<String, String> {
        // ADVANCED IMPLEMENTATION: Multi-platform machine ID generation
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
                let lines: Vec<&str> = output_str.lines().collect();
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
        use rand::{Rng, rng};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // ADVANCED IMPLEMENTATION: Secure session ID generation
        let mut rng = rng();
        let random_part: u64 = rng.gen_range(0..=u64::MAX);
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
            
        // Get process ID for additional uniqueness
        let process_id = std::process::id();
        
        // Create a hash-based session ID
        let session_data = format!("{}:{}:{}", timestamp, random_part, process_id);
        let mut hasher = sha2::Sha256::new();
        hasher.update(session_data.as_bytes());
        let result = hasher.finalize();
        
        // Convert to hex string and take first 16 characters
        let hex_string = format!("{:x}", result);
        let session_id = hex_string.chars().take(16).collect::<String>();
        
        info!("[ConfigurationBridge] Generated session ID: {}", session_id);
        Ok(format!("session-{}", session_id))
    }
}

/// Configuration status structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationStatus {
    pub is_valid: bool,
    pub last_sync: u64,
    pub configuration_keys: Vec<String>,
}

/// Tauri command to get Wind desktop configuration
#[tauri::command]
pub async fn mountain_get_wind_desktop_configuration(
    app_handle: tauri::AppHandle,
) -> Result<WindDesktopConfiguration, String> {
    debug!("[ConfigurationBridge] Tauri command: get_wind_desktop_configuration");
    
    if let Some(runtime) = app_handle.try_state::<Arc<ApplicationRunTime>>() {
        let bridge = ConfigurationBridge::new(runtime.inner().clone());
        bridge.get_wind_desktop_configuration().await
    } else {
        Err("ApplicationRunTime not found".to_string())
    }
}

/// Tauri command to update configuration from Wind
#[tauri::command]
pub async fn mountain_update_configuration_from_wind(
    app_handle: tauri::AppHandle,
    config: serde_json::Value,
) -> Result<(), String> {
    debug!("[ConfigurationBridge] Tauri command: update_configuration_from_wind");
    
    if let Some(runtime) = app_handle.try_state::<Arc<ApplicationRunTime>>() {
        let bridge = ConfigurationBridge::new(runtime.inner().clone());
        bridge.handle_wind_configuration_change(config).await
    } else {
        Err("ApplicationRunTime not found".to_string())
    }
}

/// Tauri command to synchronize configuration
#[tauri::command]
pub async fn mountain_synchronize_configuration(
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    debug!("[ConfigurationBridge] Tauri command: synchronize_configuration");
    
    if let Some(runtime) = app_handle.try_state::<Arc<ApplicationRunTime>>() {
        let bridge = ConfigurationBridge::new(runtime.inner().clone());
        bridge.synchronize_configuration().await
    } else {
        Err("ApplicationRunTime not found".to_string())
    }
}

/// Tauri command to get configuration status
#[tauri::command]
pub async fn mountain_get_configuration_status(
    app_handle: tauri::AppHandle,
) -> Result<ConfigurationStatus, String> {
    debug!("[ConfigurationBridge] Tauri command: get_configuration_status");
    
    if let Some(runtime) = app_handle.try_state::<Arc<ApplicationRunTime>>() {
        let bridge = ConfigurationBridge::new(runtime.inner().clone());
        bridge.get_configuration_status().await
    } else {
        Err("ApplicationRunTime not found".to_string())
    }
}
