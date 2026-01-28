//! # Wind Service Handlers
//! 
//! Mountain counterpart to Wind's desktop services.
//! Provides Rust implementations that mirror Wind's TypeScript service interfaces.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;
use log::{debug, error, info};
use serde_json::{Value, json};
use tauri::{AppHandle, command};

use crate::{
    ApplicationState::ApplicationState::ApplicationState,
    RunTime::ApplicationRunTime::ApplicationRunTime,
};

/// Handler for Wind's MainProcessService.invoke() calls
/// Maps Tauri IPC commands to Mountain's internal command system
#[tauri::command]
pub async fn mountain_ipc_invoke(
    app_handle: AppHandle,
    command: String,
    args: Vec<Value>,
) -> Result<Value, String> {
    debug!("[WindServiceHandlers] IPC Invoke command: {}, args: {:?}", command, args);
    
    // Get the application runtime
    let runtime = app_handle.try_state::<Arc<ApplicationRunTime>>()
        .ok_or("ApplicationRunTime not found in state".to_string())?;
    
    // Route the command based on the command name
    match command.as_str() {
        // Configuration commands
        "configuration:get" => handle_configuration_get(runtime, args).await,
        "configuration:update" => handle_configuration_update(runtime, args).await,
        
        // File system commands
        "file:read" => handle_file_read(runtime, args).await,
        "file:write" => handle_file_write(runtime, args).await,
        "file:stat" => handle_file_stat(runtime, args).await,
        
        // Storage commands
        "storage:get" => handle_storage_get(runtime, args).await,
        "storage:set" => handle_storage_set(runtime, args).await,
        
        // Environment commands
        "environment:get" => handle_environment_get(runtime, args).await,
        
        // Native host commands
        "native:showItemInFolder" => handle_show_item_in_folder(runtime, args).await,
        "native:openExternal" => handle_open_external(runtime, args).await,
        
        // Workbench commands
        "workbench:getConfiguration" => handle_workbench_configuration(runtime, args).await,
        
        // Default handler for unknown commands
        _ => {
            error!("[WindServiceHandlers] Unknown IPC command: {}", command);
            Err(format!("Unknown IPC command: {}", command))
        }
    }
}

/// Handler for configuration get requests
async fn handle_configuration_get(
    runtime: Arc<ApplicationRunTime>,
    args: Vec<Value>,
) -> Result<Value, String> {
    let key = args.get(0)
        .ok_or("Missing configuration key".to_string())?
        .as_str()
        .ok_or("Configuration key must be a string".to_string())?;
    
    // Use Mountain's configuration system
    let provider: Arc<dyn Common::Configuration::ConfigurationProvider::ConfigurationProvider> = runtime.Environment.Require();
    
    let value = provider.GetConfigurationValue(key.to_string(), Value::Null)
        .await
        .map_err(|e| format!("Failed to get configuration: {}", e))?;
    
    debug!("[WindServiceHandlers] Configuration get: {} = {:?}", key, value);
    Ok(value)
}

/// Handler for configuration update requests
async fn handle_configuration_update(
    runtime: Arc<ApplicationRunTime>,
    args: Vec<Value>,
) -> Result<Value, String> {
    let key = args.get(0)
        .ok_or("Missing configuration key".to_string())?
        .as_str()
        .ok_or("Configuration key must be a string".to_string())?;
    
    let value = args.get(1)
        .ok_or("Missing configuration value".to_string())?
        .clone();
    
    // Use Mountain's configuration system
    let provider: Arc<dyn Common::Configuration::ConfigurationProvider::ConfigurationProvider> = runtime.Environment.Require();
    
    provider.UpdateConfigurationValue(
        key.to_string(),
        value,
        Common::Configuration::DTO::ConfigurationTarget::ConfigurationTarget::User,
        Value::Null,
        None,
    )
    .await
    .map_err(|e| format!("Failed to update configuration: {}", e))?;
    
    debug!("[WindServiceHandlers] Configuration updated: {}", key);
    Ok(Value::Null)
}

/// Handler for file read requests
async fn handle_file_read(
    runtime: Arc<ApplicationRunTime>,
    args: Vec<Value>,
) -> Result<Value, String> {
    let path = args.get(0)
        .ok_or("Missing file path".to_string())?
        .as_str()
        .ok_or("File path must be a string".to_string())?;
    
    // Use Mountain's file system provider
    let provider: Arc<dyn Common::FileSystem::FileSystemProvider::FileSystemProvider> = runtime.Environment.Require();
    
    let content = provider.ReadFile(path.to_string())
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    debug!("[WindServiceHandlers] File read: {} ({} bytes)", path, content.len());
    Ok(json!(content))
}

/// Handler for file write requests
async fn handle_file_write(
    runtime: Arc<ApplicationRunTime>,
    args: Vec<Value>,
) -> Result<Value, String> {
    let path = args.get(0)
        .ok_or("Missing file path".to_string())?
        .as_str()
        .ok_or("File path must be a string".to_string())?;
    
    let content = args.get(1)
        .ok_or("Missing file content".to_string())?
        .as_str()
        .ok_or("File content must be a string".to_string())?;
    
    // Use Mountain's file system provider
    let provider: Arc<dyn Common::FileSystem::FileSystemProvider::FileSystemProvider> = runtime.Environment.Require();
    
    provider.WriteFile(path.to_string(), content.as_bytes().to_vec(), true, true)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    debug!("[WindServiceHandlers] File written: {} ({} bytes)", path, content.len());
    Ok(Value::Null)
}

/// Handler for file stat requests
async fn handle_file_stat(
    runtime: Arc<ApplicationRunTime>,
    args: Vec<Value>,
) -> Result<Value, String> {
    let path = args.get(0)
        .ok_or("Missing file path".to_string())?
        .as_str()
        .ok_or("File path must be a string".to_string())?;
    
    // Use Mountain's file system provider
    let provider: Arc<dyn Common::FileSystem::FileSystemProvider::FileSystemProvider> = runtime.Environment.Require();
    
    let stats = provider.StatFile(path.to_string())
        .await
        .map_err(|e| format!("Failed to stat file: {}", e))?;
    
    debug!("[WindServiceHandlers] File stat: {}", path);
    Ok(json!(stats))
}

/// Handler for storage get requests
async fn handle_storage_get(
    runtime: Arc<ApplicationRunTime>,
    args: Vec<Value>,
) -> Result<Value, String> {
    let key = args.get(0)
        .ok_or("Missing storage key".to_string())?
        .as_str()
        .ok_or("Storage key must be a string".to_string())?;
    
    // Use Mountain's storage provider
    let provider: Arc<dyn Common::Storage::StorageProvider::StorageProvider> = runtime.Environment.Require();
    
    let value = provider.GetStorageItem(key.to_string())
        .await
        .map_err(|e| format!("Failed to get storage item: {}", e))?;
    
    debug!("[WindServiceHandlers] Storage get: {}", key);
    Ok(value)
}

/// Handler for storage set requests
async fn handle_storage_set(
    runtime: Arc<ApplicationRunTime>,
    args: Vec<Value>,
) -> Result<Value, String> {
    let key = args.get(0)
        .ok_or("Missing storage key".to_string())?
        .as_str()
        .ok_or("Storage key must be a string".to_string())?;
    
    let value = args.get(1)
        .ok_or("Missing storage value".to_string())?
        .clone();
    
    // Use Mountain's storage provider
    let provider: Arc<dyn Common::Storage::StorageProvider::StorageProvider> = runtime.Environment.Require();
    
    provider.SetStorageItem(key.to_string(), value)
        .await
        .map_err(|e| format!("Failed to set storage item: {}", e))?;
    
    debug!("[WindServiceHandlers] Storage set: {}", key);
    Ok(Value::Null)
}

/// Handler for environment get requests
async fn handle_environment_get(
    runtime: Arc<ApplicationRunTime>,
    args: Vec<Value>,
) -> Result<Value, String> {
    let key = args.get(0)
        .ok_or("Missing environment key".to_string())?
        .as_str()
        .ok_or("Environment key must be a string".to_string())?;
    
    // Use Mountain's environment service
    let provider: Arc<dyn Common::Environment::EnvironmentProvider::EnvironmentProvider> = runtime.Environment.Require();
    
    let value = provider.GetEnvironmentVariable(key.to_string())
        .await
        .map_err(|e| format!("Failed to get environment variable: {}", e))?;
    
    debug!("[WindServiceHandlers] Environment get: {}", key);
    Ok(json!(value))
}

/// Handler for showing items in folder
async fn handle_show_item_in_folder(
    _runtime: Arc<ApplicationRunTime>,
    args: Vec<Value>,
) -> Result<Value, String> {
    let path = args.get(0)
        .ok_or("Missing file path".to_string())?
        .as_str()
        .ok_or("File path must be a string".to_string())?;
    
    // TODO: Implement native file system integration
    debug!("[WindServiceHandlers] Show item in folder: {}", path);
    Ok(Value::Null)
}

/// Handler for opening external URLs
async fn handle_open_external(
    _runtime: Arc<ApplicationRunTime>,
    args: Vec<Value>,
) -> Result<Value, String> {
    let url = args.get(0)
        .ok_or("Missing URL".to_string())?
        .as_str()
        .ok_or("URL must be a string".to_string())?;
    
    // TODO: Implement native URL opening
    debug!("[WindServiceHandlers] Open external: {}", url);
    Ok(Value::Null)
}

/// Handler for workbench configuration requests
async fn handle_workbench_configuration(
    runtime: Arc<ApplicationRunTime>,
    _args: Vec<Value>,
) -> Result<Value, String> {
    // Get the complete workbench configuration
    let provider: Arc<dyn Common::Configuration::ConfigurationProvider::ConfigurationProvider> = runtime.Environment.Require();
    
    let config = provider.GetConfiguration(None, Value::Null)
        .await
        .map_err(|e| format!("Failed to get workbench configuration: {}", e))?;
    
    debug!("[WindServiceHandlers] Workbench configuration retrieved");
    Ok(config)
}

/// Register all Wind IPC command handlers
pub fn register_wind_ipc_handlers(app_handle: &tauri::AppHandle) -> Result<(), String> {
    info!("[WindServiceHandlers] Registering Wind IPC command handlers");
    
    // Note: These handlers are automatically registered when included in the
    // Tauri invoke_handler macro in the main binary
    
    Ok(())
}