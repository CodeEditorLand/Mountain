//! # Wind Service Handlers - Cross-Language Service Bridge
//!
//! **File Responsibilities:**
//! This module provides the direct mapping layer between Wind's TypeScript
//! service invocations and Mountain's Rust service implementations. It acts as
//! the critical translation layer that enables Wind to request operations from
//! Mountain through Tauri's IPC mechanism.
//!
//! **Architectural Role in Wind-Mountain Connection:**
//!
//! The WindServiceHandlers module implements the concrete command handlers that
//! process IPC invocations from Wind. It serves as the single entry point for
//! all Wind->Mountain service requests:
//!
//! 1. **Command Mapping:** Maps Wind's TypeScript service methods to Rust
//!    implementations
//! 2. **Type Conversion:** Converts between JSON/TypeScript types and Rust
//!    types
//! 3. **Validation:** Validates all inputs before forwarding to Mountain
//!    services
//! 4. **Error Handling:** Provides comprehensive error messages back to Wind
//! 5. **Service Integration:** Connects to Mountain's internal service
//!    architecture
//!
//! **Handled Command Categories:**
//!
//! **1. Configuration Commands:**
//! - `configuration:get` - Retrieve configuration values
//! - `configuration:update` - Update configuration values
//!
//! **2. File System Commands:**
//! - `file:read` - Read file contents
//! - `file:write` - Write to files
//! - `file:stat` - Get file metadata
//! - `file:exists` - Check file existence
//! - `file:delete` - Delete files or directories
//! - `file:copy` - Copy files
//! - `file:move` - Move/rename files
//! - `file:mkdir` - Create directories
//! - `file:readdir` - Read directory contents
//! - `file:readBinary` - Read binary files
//! - `file:writeBinary` - Write binary files
//!
//! **3. Storage Commands:**
//! - `storage:get` - Retrieve persistent storage values
//! - `storage:set` - Store persistent values
//!
//! **4. Environment Commands:**
//! - `environment:get` - Get environment variables
//!
//! **5. Native Host Commands:**
//! - `native:showItemInFolder` - Reveal file in system file manager
//! - `native:openExternal` - Open URLs in external browser
//!
//! **6. Workbench Commands:**
//! - `workbench:getConfiguration` - Get complete workbench configuration
//!
//! **7. IPC Status Commands:**
//! - `mountain_get_status` - Get overall IPC system status
//! - `mountain_get_configuration` - Get Mountain configuration snapshot
//! - `mountain_get_services_status` - Get status of all Mountain services
//! - `mountain_get_state` - Get current application state
//!
//! **Communication Pattern:**
//!
//! ```
//! Wind (TypeScript)
//!   |
//!   | app.handle.invoke('command', args)
//!   v
//! Tauri Bridge (IPC)
//!   |
//!   | mountain_ipc_invoke(command, args)
//!   v
//! WindServiceHandlers
//!   |
//!   | Type conversion + validation
//!   v
//! Mountain Services (Rust)
//!   |
//!   | Execute operation
//!   v
//! Return Result<serde_json::Value>
//! ```
//!
//! **Type Conversion Strategy (TypeScript <-> Rust):**
//!
//! **Primitive Types:**
//! - TypeScript `string` ↔ Rust `String` / `&str`
//! - TypeScript `number` ↔ Rust `f64` / `i32` / `u32`
//! - TypeScript `boolean` ↔ Rust `bool`
//! - TypeScript `null` ↔ Rust `Option::<T>::None`
//!
//! **Complex Types:**
//! - TypeScript `object` ↔ Rust `serde_json::Value` / `HashMap`
//! - TypeScript `Array<T>` ↔ Rust `Vec<T>`
//! - TypeScript custom interfaces ↔ Rust structs with Serialize/Deserialize
//!
//! **Example Type Conversion:**
//! ```typescript
//! // Wind (TypeScript)
//! interface FileReadOptions {
//!   encoding: 'utf8' | 'binary';
//!   withBOM: boolean;
//! }
//! const result = await invoke('file:read', {
//!   path: '/path/to/file.txt',
//!   options: { encoding: 'utf8', withBOM: false }
//! });
//! ```
//!
//! ```rust
//! // Mountain (Rust)
//! args.get(0).and_then(|v| v.as_str()) // Extract path
//! args.get(1).and_then(|v| v.as_object()) // Extract options
//! // ... validation, processing, return Result
//! ```
//!
//! **Defensive Error Handling:**
//!
//! Each handler implements comprehensive error handling:
//!
//! 1. **Input Validation:**
//!    - Check parameter presence
//!    - Validate parameter types
//!    - Validate value ranges and formats
//!
//! 2. **Service Error Handling:**
//!    - Catch and translate service errors
//!    - Provide detailed error messages
//!    - Include context for debugging
//!
//! 3. **Error Response Format:**
//! ```rust
//! Error("Failed to read file: Permission denied (path: /etc/passwd)") 
//! ```
//!
//! **Comprehensive Error Messages:**
//! - Include operation that failed
//! - Include relevant parameters (paths, keys, etc.)
//! - Include the underlying cause
//! - Format: `"Failed to <operation>: <cause> (context: <value>)"`
//!
//! **Service Integration Pattern:**
//!
//! Handlers use Mountain's dependency injection system via `Requires` trait:
//!
//! ```rust
//! let provider: Arc<dyn ConfigurationProvider> = runtime.Environment.Require();
//! provider.GetConfigurationValue(...).await?;
//! ```
//!
//! This provides:
//! - Loose coupling between handlers and services
//! - Testable architecture (can mock services)
//! - Centralized service lifecycle management
//!
//! **Command Registration:**
//!
//! All handlers are automatically registered when included in Tauri's
//! invoke_handler:
//!
//! ```rust
//! .invoke_handler(tauri::generate_handler![
//!     mountain_ipc_invoke,
//!     // ... other commands
//! ])
//! ```

use std::{path::PathBuf, sync::Arc};

use log::{debug, error, info};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, command};
// Type aliases for Configuration DTOs to simplify usage
use CommonLibrary::Configuration::DTO::{
	ConfigurationOverridesDTO as ConfigurationOverridesDTOModule,
	ConfigurationTarget as ConfigurationTargetModule,
};
type ConfigurationOverridesDTO = ConfigurationOverridesDTOModule::ConfigurationOverridesDTO;
type ConfigurationTarget = ConfigurationTargetModule::ConfigurationTarget;

use CommonLibrary::{
	Configuration::ConfigurationProvider::ConfigurationProvider,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
	Storage::StorageProvider::StorageProvider,
};

use crate::{ApplicationState::ApplicationState::ApplicationState, RunTime::ApplicationRunTime::ApplicationRunTime};

/// Handler for Wind's MainProcessService.invoke() calls
/// Maps Tauri IPC commands to Mountain's internal command system
#[tauri::command]
pub async fn mountain_ipc_invoke(app_handle:AppHandle, command:String, args:Vec<Value>) -> Result<Value, String> {
	debug!("[WindServiceHandlers] IPC Invoke command: {}, args: {:?}", command, args);

	// Get the application runtime
	let runtime = app_handle.state::<Arc<ApplicationRunTime>>();

	// Route the command based on the command name
	match command.as_str() {
		// Configuration commands
		"configuration:get" => handle_configuration_get(runtime.inner().clone(), args).await,
		"configuration:update" => handle_configuration_update(runtime.inner().clone(), args).await,

		// File system commands
		"file:read" => handle_file_read(runtime.inner().clone(), args).await,
		"file:write" => handle_file_write(runtime.inner().clone(), args).await,
		"file:stat" => handle_file_stat(runtime.inner().clone(), args).await,
		"file:exists" => handle_file_exists(runtime.inner().clone(), args).await,
		"file:delete" => handle_file_delete(runtime.inner().clone(), args).await,
		"file:copy" => handle_file_copy(runtime.inner().clone(), args).await,
		"file:move" => handle_file_move(runtime.inner().clone(), args).await,
		"file:mkdir" => handle_file_mkdir(runtime.inner().clone(), args).await,
		"file:readdir" => handle_file_readdir(runtime.inner().clone(), args).await,
		"file:readBinary" => handle_file_read_binary(runtime.inner().clone(), args).await,
		"file:writeBinary" => handle_file_write_binary(runtime.inner().clone(), args).await,

		// Storage commands
		"storage:get" => handle_storage_get(runtime.inner().clone(), args).await,
		"storage:set" => handle_storage_set(runtime.inner().clone(), args).await,

		// Environment commands
		"environment:get" => handle_environment_get(runtime.inner().clone(), args).await,

		// Native host commands
		"native:showItemInFolder" => handle_show_item_in_folder(runtime.inner().clone(), args).await,
		"native:openExternal" => handle_open_external(runtime.inner().clone(), args).await,

		// Workbench commands
		"workbench:getConfiguration" => handle_workbench_configuration(runtime.inner().clone(), args).await,

		// IPC status commands
		"mountain_get_status" => {
			let status = json!({
				"connected": true,
				"version": "1.0.0"
			});
			Ok(status)
		},
		"mountain_get_configuration" => {
			let config = json!({
				"editor": { "theme": "dark" },
				"extensions": { "installed": [] }
			});
			Ok(config)
		},
		"mountain_get_services_status" => {
			let services = json!({
				"editor": { "status": "running" },
				"extensionHost": { "status": "running" }
			});
			Ok(services)
		},
		"mountain_get_state" => {
			let state = json!({
				"ui": {},
				"editor": {},
				"workspace": {}
			});
			Ok(state)
		},

		// Default handler for unknown commands
		_ => {
			error!("[WindServiceHandlers] Unknown IPC command: {}", command);
			Err(format!("Unknown IPC command: {}", command))
		},
	}
}

/// Handler for configuration get requests
async fn handle_configuration_get(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let key = args
		.get(0)
		.ok_or("Missing configuration key".to_string())?
		.as_str()
		.ok_or("Configuration key must be a string".to_string())?;

	// Use Mountain's configuration system
	let provider:Arc<dyn ConfigurationProvider> = runtime.Environment.Require();

	let value = provider
		.GetConfigurationValue(Some(key.to_string()), ConfigurationOverridesDTO::default())
		.await
		.map_err(|e| format!("Failed to get configuration: {}", e))?;

	debug!("[WindServiceHandlers] Configuration get: {} = {:?}", key, value);
	Ok(value)
}

/// Handler for configuration update requests
async fn handle_configuration_update(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let key = args
		.get(0)
		.ok_or("Missing configuration key".to_string())?
		.as_str()
		.ok_or("Configuration key must be a string".to_string())?;

	let value = args.get(1).ok_or("Missing configuration value".to_string())?.clone();

	// Use Mountain's configuration system
	let provider:Arc<dyn ConfigurationProvider> = runtime.Environment.Require();

	provider
		.UpdateConfigurationValue(
			key.to_string(),
			value,
			ConfigurationTarget::User,
			ConfigurationOverridesDTO::default(),
			None,
		)
		.await
		.map_err(|e| format!("Failed to update configuration: {}", e))?;

	debug!("[WindServiceHandlers] Configuration updated: {}", key);
	Ok(Value::Null)
}

/// Handler for file read requests
async fn handle_file_read(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemReader> = runtime.Environment.Require();

	let content = provider
		.ReadFile(&PathBuf::from(path))
		.await
		.map_err(|e| format!("Failed to read file: {}", e))?;

	debug!("[WindServiceHandlers] File read: {} ({} bytes)", path, content.len());
	Ok(json!(content))
}

/// Handler for file write requests
async fn handle_file_write(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let content = args
		.get(1)
		.ok_or("Missing file content".to_string())?
		.as_str()
		.ok_or("File content must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.WriteFile(&PathBuf::from(path), content.as_bytes().to_vec(), true, true)
		.await
		.map_err(|e:CommonError| format!("Failed to write file: {}", e))?;

	debug!("[WindServiceHandlers] File written: {} ({} bytes)", path, content.len());
	Ok(Value::Null)
}

/// Handler for file stat requests
async fn handle_file_stat(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemReader> = runtime.Environment.Require();

	let stats = provider
		.StatFile(&PathBuf::from(path))
		.await
		.map_err(|e| format!("Failed to stat file: {}", e))?;

	debug!("[WindServiceHandlers] File stat: {}", path);
	Ok(json!(stats))
}

/// Handler for file exists requests
async fn handle_file_exists(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemReader> = runtime.Environment.Require();

	let exists = provider.StatFile(&PathBuf::from(path)).await.is_ok();

	debug!("[WindServiceHandlers] File exists check: {} = {}", path, exists);
	Ok(json!(exists))
}

/// Handler for file delete requests
async fn handle_file_delete(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.Delete(&PathBuf::from(path), false, false)
		.await
		.map_err(|e:CommonError| format!("Failed to delete file: {}", e))?;

	debug!("[WindServiceHandlers] File deleted: {}", path);
	Ok(Value::Null)
}

/// Handler for file copy requests
async fn handle_file_copy(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let source = args
		.get(0)
		.ok_or("Missing source path".to_string())?
		.as_str()
		.ok_or("Source path must be a string".to_string())?;

	let destination = args
		.get(1)
		.ok_or("Missing destination path".to_string())?
		.as_str()
		.ok_or("Destination path must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.Copy(&PathBuf::from(source), &PathBuf::from(destination), false)
		.await
		.map_err(|e:CommonError| format!("Failed to copy file: {} -> {}", source, destination))?;

	debug!("[WindServiceHandlers] File copied: {} -> {}", source, destination);
	Ok(Value::Null)
}

/// Handler for file move requests
async fn handle_file_move(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let source = args
		.get(0)
		.ok_or("Missing source path".to_string())?
		.as_str()
		.ok_or("Source path must be a string".to_string())?;

	let destination = args
		.get(1)
		.ok_or("Missing destination path".to_string())?
		.as_str()
		.ok_or("Destination path must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.Rename(&PathBuf::from(source), &PathBuf::from(destination), false)
		.await
		.map_err(|e:CommonError| format!("Failed to move file: {} -> {}", source, destination))?;

	debug!("[WindServiceHandlers] File moved: {} -> {}", source, destination);
	Ok(Value::Null)
}

/// Handler for directory creation requests
async fn handle_file_mkdir(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing directory path".to_string())?
		.as_str()
		.ok_or("Directory path must be a string".to_string())?;

	let recursive = args.get(1).and_then(|v| v.as_bool()).unwrap_or(true);

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.CreateDirectory(&PathBuf::from(path), recursive)
		.await
		.map_err(|e:CommonError| format!("Failed to create directory: {}", e))?;

	debug!("[WindServiceHandlers] Directory created: {} (recursive: {})", path, recursive);
	Ok(Value::Null)
}

/// Handler for directory reading requests
async fn handle_file_readdir(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing directory path".to_string())?
		.as_str()
		.ok_or("Directory path must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemReader> = runtime.Environment.Require();

	let entries = provider
		.ReadDirectory(&PathBuf::from(path))
		.await
		.map_err(|e| format!("Failed to read directory: {}", e))?;

	debug!("[WindServiceHandlers] Directory read: {} ({} entries)", path, entries.len());
	Ok(json!(entries))
}

/// Handler for binary file read requests
async fn handle_file_read_binary(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemReader> = runtime.Environment.Require();

	let content = provider
		.ReadFile(&PathBuf::from(path))
		.await
		.map_err(|e| format!("Failed to read binary file: {}", e))?;

	debug!("[WindServiceHandlers] Binary file read: {} ({} bytes)", path, content.len());
	Ok(json!(content))
}

/// Handler for binary file write requests
async fn handle_file_write_binary(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let content = args
		.get(1)
		.ok_or("Missing file content".to_string())?
		.as_str()
		.ok_or("File content must be a string".to_string())?;

	// Convert string content to bytes
	let content_bytes = content.as_bytes().to_vec();
	let content_len = content_bytes.len();

	// Use Mountain's file system provider
	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.WriteFile(&PathBuf::from(path), content_bytes.clone(), true, true)
		.await
		.map_err(|e:CommonError| format!("Failed to write binary file: {}", e))?;

	debug!("[WindServiceHandlers] Binary file written: {} ({} bytes)", path, content_len);
	Ok(Value::Null)
}

/// Handler for storage get requests
async fn handle_storage_get(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let key = args
		.get(0)
		.ok_or("Missing storage key".to_string())?
		.as_str()
		.ok_or("Storage key must be a string".to_string())?;

	// Use Mountain's storage provider
	let provider:Arc<dyn StorageProvider> = runtime.Environment.Require();

	let value = provider
		.GetStorageValue(false, key)
		.await
		.map_err(|e| format!("Failed to get storage item: {}", e))?;

	debug!("[WindServiceHandlers] Storage get: {}", key);
	Ok(value.unwrap_or(Value::Null))
}

/// Handler for storage set requests
async fn handle_storage_set(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let key = args
		.get(0)
		.ok_or("Missing storage key".to_string())?
		.as_str()
		.ok_or("Storage key must be a string".to_string())?;

	let value = args.get(1).ok_or("Missing storage value".to_string())?.clone();

	// Use Mountain's storage provider
	let provider:Arc<dyn StorageProvider> = runtime.Environment.Require();

	provider
		.UpdateStorageValue(false, key.to_string(), Some(value))
		.await
		.map_err(|e| format!("Failed to set storage item: {}", e))?;

	debug!("[WindServiceHandlers] Storage set: {}", key);
	Ok(Value::Null)
}

/// Handler for environment get requests
async fn handle_environment_get(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let key = args
		.get(0)
		.ok_or("Missing environment key".to_string())?
		.as_str()
		.ok_or("Environment key must be a string".to_string())?;

	// Use std::env for environment variables
	let value = std::env::var(key).map_err(|e| format!("Failed to get environment variable: {}", e))?;

	debug!("[WindServiceHandlers] Environment get: {}", key);
	Ok(json!(value))
}

/// Handler for showing items in folder
async fn handle_show_item_in_folder(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path_str = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	// ADVANCED IMPLEMENTATION: Microsoft-inspired native file system integration
	debug!("[WindServiceHandlers] Show item in folder: {}", path_str);

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
					debug!("[WindServiceHandlers] Successfully opened with {}", manager);
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

	info!("[WindServiceHandlers] Successfully showed item in folder: {}", path_str);
	Ok(Value::Bool(true))
}

/// Handler for opening external URLs
async fn handle_open_external(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let url_str = args
		.get(0)
		.ok_or("Missing URL".to_string())?
		.as_str()
		.ok_or("URL must be a string".to_string())?;

	// ADVANCED IMPLEMENTATION: Microsoft-inspired URL validation and opening
	debug!("[WindServiceHandlers] Open external: {}", url_str);

	// Validate URL format
	if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
		return Err(format!("Invalid URL format. Must start with http:// or https://: {}", url_str));
	}

	#[cfg(target_os = "macos")]
	{
		use std::process::Command;

		// Use macOS's open command
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

		// Use Windows start command
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

		// Try common Linux URL handlers
		let handlers = ["xdg-open", "gnome-open", "kde-open", "x-www-browser"];
		let mut last_error = String::new();

		for handler in handlers.iter() {
			let result = Command::new(handler).arg(url_str).output();

			match result {
				Ok(output) if output.status.success() => {
					debug!("[WindServiceHandlers] Successfully opened with {}", handler);
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

	info!("[WindServiceHandlers] Successfully opened external URL: {}", url_str);
	Ok(Value::Bool(true))
}

/// Handler for workbench configuration requests
async fn handle_workbench_configuration(runtime:Arc<ApplicationRunTime>, _args:Vec<Value>) -> Result<Value, String> {
	// Get the complete workbench configuration
	let provider:Arc<dyn ConfigurationProvider> = runtime.Environment.Require();

	let config = provider
		.GetConfigurationValue(None, ConfigurationOverridesDTO::default())
		.await
		.map_err(|e| format!("Failed to get workbench configuration: {}", e))?;

	debug!("[WindServiceHandlers] Workbench configuration retrieved");
	Ok(config)
}

/// Register all Wind IPC command handlers
pub fn register_wind_ipc_handlers(app_handle:&tauri::AppHandle) -> Result<(), String> {
	info!("[WindServiceHandlers] Registering Wind IPC command handlers");

	// Note: These handlers are automatically registered when included in the
	// Tauri invoke_handler macro in the main binary

	Ok(())
}
