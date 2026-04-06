#![allow(unused_variables, dead_code)]

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
//! ```text
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
//! ```text
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
//! ```text
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
use tauri::{AppHandle, Manager};
// Type aliases for Configuration DTOs to simplify usage
use CommonLibrary::Configuration::DTO::{
	ConfigurationOverridesDTO as ConfigurationOverridesDTOModule,
	ConfigurationTarget as ConfigurationTargetModule,
};
type ConfigurationOverridesDTO = ConfigurationOverridesDTOModule::ConfigurationOverridesDTO;
type ConfigurationTarget = ConfigurationTargetModule::ConfigurationTarget;

use CommonLibrary::{
	Command::CommandExecutor::CommandExecutor,
	Configuration::ConfigurationProvider::ConfigurationProvider,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	ExtensionManagement::ExtensionManagementService::ExtensionManagementService,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
	Storage::StorageProvider::StorageProvider,
};

use crate::{
	ApplicationState::DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

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

		// Command registry commands
		"commands:execute" => handle_commands_execute(runtime.inner().clone(), args).await,
		"commands:getAll" => handle_commands_get_all(runtime.inner().clone()).await,

		// Extension host commands
		"extensions:getAll" => handle_extensions_get_all(runtime.inner().clone()).await,
		"extensions:get" => handle_extensions_get(runtime.inner().clone(), args).await,
		"extensions:isActive" => handle_extensions_is_active(runtime.inner().clone(), args).await,

		// Terminal commands
		"terminal:create" => handle_terminal_create(runtime.inner().clone(), args).await,
		"terminal:sendText" => handle_terminal_send_text(runtime.inner().clone(), args).await,
		"terminal:dispose" => handle_terminal_dispose(runtime.inner().clone(), args).await,
		"terminal:show" => handle_terminal_show(runtime.inner().clone(), args).await,
		"terminal:hide" => handle_terminal_hide(runtime.inner().clone(), args).await,

		// Output channel commands
		"output:create" => handle_output_create(app_handle.clone(), args).await,
		"output:append" => handle_output_append(app_handle.clone(), args).await,
		"output:appendLine" => handle_output_append_line(app_handle.clone(), args).await,
		"output:clear" => handle_output_clear(app_handle.clone(), args).await,
		"output:show" => handle_output_show(app_handle.clone(), args).await,

		// TextFile commands
		"textFile:read" => handle_textfile_read(runtime.inner().clone(), args).await,
		"textFile:write" => handle_textfile_write(runtime.inner().clone(), args).await,
		"textFile:save" => handle_textfile_save(runtime.inner().clone(), args).await,

		// Storage commands
		"storage:delete" => handle_storage_delete(runtime.inner().clone(), args).await,
		"storage:keys" => handle_storage_keys(runtime.inner().clone()).await,

		// Notification commands (emit sky:// events for Sky to render)
		"notification:show" => handle_notification_show(app_handle.clone(), args).await,
		"notification:showProgress" => handle_notification_show_progress(app_handle.clone(), args).await,
		"notification:updateProgress" => handle_notification_update_progress(app_handle.clone(), args).await,
		"notification:endProgress" => handle_notification_end_progress(app_handle.clone(), args).await,

		// Progress commands
		"progress:begin" => handle_progress_begin(app_handle.clone(), args).await,
		"progress:report" => handle_progress_report(app_handle.clone(), args).await,
		"progress:end" => handle_progress_end(app_handle.clone(), args).await,

		// QuickInput commands (routed through UserInterfaceProvider in CocoonService)
		"quickInput:showQuickPick" => handle_quick_input_show_quick_pick(runtime.inner().clone(), args).await,
		"quickInput:showInputBox" => handle_quick_input_show_input_box(runtime.inner().clone(), args).await,

		// Workspaces commands
		"workspaces:getFolders" => handle_workspaces_get_folders(runtime.inner().clone()).await,
		"workspaces:addFolder" => handle_workspaces_add_folder(runtime.inner().clone(), args).await,
		"workspaces:removeFolder" => handle_workspaces_remove_folder(runtime.inner().clone(), args).await,
		"workspaces:getName" => handle_workspaces_get_name(runtime.inner().clone()).await,

		// Themes commands
		"themes:getActive" => handle_themes_get_active(runtime.inner().clone()).await,
		"themes:list" => handle_themes_list(runtime.inner().clone()).await,
		"themes:set" => handle_themes_set(runtime.inner().clone(), args).await,

		// Search commands
		"search:findInFiles" => handle_search_find_in_files(runtime.inner().clone(), args).await,
		"search:findFiles" => handle_search_find_files(runtime.inner().clone(), args).await,

		// Decorations commands
		"decorations:get" => handle_decorations_get(runtime.inner().clone(), args).await,
		"decorations:getMany" => handle_decorations_get_many(runtime.inner().clone(), args).await,
		"decorations:set" => handle_decorations_set(runtime.inner().clone(), args).await,
		"decorations:clear" => handle_decorations_clear(runtime.inner().clone(), args).await,

		// WorkingCopy commands
		"workingCopy:isDirty" => handle_working_copy_is_dirty(runtime.inner().clone(), args).await,
		"workingCopy:setDirty" => handle_working_copy_set_dirty(runtime.inner().clone(), args).await,
		"workingCopy:getAllDirty" => handle_working_copy_get_all_dirty(runtime.inner().clone()).await,
		"workingCopy:getDirtyCount" => handle_working_copy_get_dirty_count(runtime.inner().clone()).await,

		// Keybinding commands
		"keybinding:add" => handle_keybinding_add(runtime.inner().clone(), args).await,
		"keybinding:remove" => handle_keybinding_remove(runtime.inner().clone(), args).await,
		"keybinding:lookup" => handle_keybinding_lookup(runtime.inner().clone(), args).await,
		"keybinding:getAll" => handle_keybinding_get_all(runtime.inner().clone()).await,

		// Lifecycle commands
		"lifecycle:getPhase" => handle_lifecycle_get_phase(runtime.inner().clone()).await,
		"lifecycle:whenPhase" => handle_lifecycle_when_phase(runtime.inner().clone(), args).await,
		"lifecycle:requestShutdown" => handle_lifecycle_request_shutdown(app_handle.clone()).await,

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

	// IMPLEMENTATION: Microsoft-inspired native file system integration
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

	// IMPLEMENTATION: Microsoft-inspired URL validation and opening
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

// ============================================================================
// Terminal Handlers
// ============================================================================

/// Create a new PTY terminal via TerminalProvider.
async fn handle_terminal_create(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

	let Options = args.first().cloned().unwrap_or(Value::Null);
	runtime
		.Environment
		.CreateTerminal(Options)
		.await
		.map_err(|Error| format!("terminal:create failed: {}", Error))
}

/// Write text to PTY stdin via TerminalProvider.
async fn handle_terminal_send_text(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

	let TerminalId = args
		.first()
		.and_then(|V| V.as_u64())
		.ok_or_else(|| "terminal:sendText requires terminal_id as first argument".to_string())?;
	let Text = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	runtime
		.Environment
		.SendTextToTerminal(TerminalId, Text)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:sendText failed: {}", Error))
}

/// Dispose a terminal via TerminalProvider.
async fn handle_terminal_dispose(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

	let TerminalId = args
		.first()
		.and_then(|V| V.as_u64())
		.ok_or_else(|| "terminal:dispose requires terminal_id as first argument".to_string())?;

	runtime
		.Environment
		.DisposeTerminal(TerminalId)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:dispose failed: {}", Error))
}

/// Show a terminal in the UI.
async fn handle_terminal_show(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

	let TerminalId = args.first().and_then(|V| V.as_u64()).unwrap_or(0);
	let PreserveFocus = args.get(1).and_then(|V| V.as_bool()).unwrap_or(false);

	runtime
		.Environment
		.ShowTerminal(TerminalId, PreserveFocus)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:show failed: {}", Error))
}

/// Hide a terminal.
async fn handle_terminal_hide(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Terminal::TerminalProvider::TerminalProvider;

	let TerminalId = args.first().and_then(|V| V.as_u64()).unwrap_or(0);

	runtime
		.Environment
		.HideTerminal(TerminalId)
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("terminal:hide failed: {}", Error))
}

// ============================================================================
// Output Channel Handlers
// ============================================================================

/// Create a named output channel. Returns the channel name as its handle.
async fn handle_output_create(_app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	let ChannelName = args.first().and_then(|V| V.as_str()).unwrap_or("Output").to_string();
	info!("[WindServiceHandlers] output:create channel='{}'", ChannelName);
	// Sky/frontend creates the channel panel on the `sky://output/create` event
	Ok(json!({ "channelName": ChannelName }))
}

/// Append text to an output channel.
async fn handle_output_append(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let ChannelName = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Text = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = app_handle.emit("sky://output/append", json!({ "channel": ChannelName, "text": Text }));
	Ok(Value::Null)
}

/// Append a line to an output channel (text + newline).
async fn handle_output_append_line(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let ChannelName = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Text = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Line = format!("{}\n", Text);

	let _ = app_handle.emit("sky://output/append", json!({ "channel": ChannelName, "text": Line }));
	Ok(Value::Null)
}

/// Clear an output channel.
async fn handle_output_clear(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let ChannelName = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let _ = app_handle.emit("sky://output/clear", json!({ "channel": ChannelName }));
	Ok(Value::Null)
}

/// Show an output channel panel.
async fn handle_output_show(app_handle:AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let ChannelName = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let _ = app_handle.emit("sky://output/show", json!({ "channel": ChannelName }));
	Ok(Value::Null)
}

// ============================================================================
// TextFile Handlers
// ============================================================================

/// Read a text file from disk.
async fn handle_textfile_read(_runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Path = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "textFile:read requires path as first argument".to_string())?;

	tokio::fs::read_to_string(Path)
		.await
		.map(Value::String)
		.map_err(|Error| format!("textFile:read failed: {}", Error))
}

/// Write text to a file on disk.
async fn handle_textfile_write(_runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Path = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "textFile:write requires path as first argument".to_string())?;
	let Content = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	tokio::fs::write(Path, Content.as_bytes())
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("textFile:write failed: {}", Error))
}

/// Save a document — forward save intent to Sky frontend.
async fn handle_textfile_save(_runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	// Actual disk write happens via textFile:write; this is a UI-dirty-state hint.
	let _Uri = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	info!("[WindServiceHandlers] textFile:save uri={:?}", _Uri);
	Ok(Value::Null)
}

/// Register all Wind IPC command handlers
pub fn register_wind_ipc_handlers(app_handle:&tauri::AppHandle) -> Result<(), String> {
	info!("[WindServiceHandlers] Registering Wind IPC command handlers");

	// Note: These handlers are automatically registered when included in the
	// Tauri invoke_handler macro in the main binary

	Ok(())
}

// ============================================================================
// Command Registry Handlers
// ============================================================================

/// Execute a command by ID, dispatching to Mountain's CommandExecutor.
async fn handle_commands_execute(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let CommandId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "commands:execute requires string command_id as first argument".to_string())?
		.to_string();

	let Argument = args.get(1).cloned().unwrap_or(Value::Null);

	debug!("[WindServiceHandlers] commands:execute id={}", CommandId);

	runtime
		.Environment
		.ExecuteCommand(CommandId, Argument)
		.await
		.map_err(|Error| format!("commands:execute failed: {}", Error))
}

/// Return all registered command IDs from Mountain's CommandRegistry.
async fn handle_commands_get_all(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Commands = runtime
		.Environment
		.GetAllCommands()
		.await
		.map_err(|Error| format!("commands:getAll failed: {}", Error))?;

	Ok(json!(Commands))
}

// ============================================================================
// Extension Host Handlers
// ============================================================================

/// Return metadata for all scanned extensions.
async fn handle_extensions_get_all(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Extensions = runtime
		.Environment
		.GetExtensions()
		.await
		.map_err(|Error| format!("extensions:getAll failed: {}", Error))?;

	Ok(json!(Extensions))
}

/// Return metadata for a single extension by ID.
async fn handle_extensions_get(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Id = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "extensions:get requires string id as first argument".to_string())?
		.to_string();

	let Extension = runtime
		.Environment
		.GetExtension(Id)
		.await
		.map_err(|Error| format!("extensions:get failed: {}", Error))?;

	Ok(Extension.unwrap_or(Value::Null))
}

/// Check whether an extension is currently active (scanned and present).
async fn handle_extensions_is_active(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Id = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "extensions:isActive requires string id as first argument".to_string())?
		.to_string();

	let Extension = runtime
		.Environment
		.GetExtension(Id)
		.await
		.map_err(|Error| format!("extensions:isActive failed: {}", Error))?;

	Ok(json!(Extension.is_some()))
}

// ============================================================================
// Storage handlers
// ============================================================================

/// Delete a persistent storage key.
async fn handle_storage_delete(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Storage::StorageProvider::StorageProvider;

	let Key = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("storage:delete requires key as first argument".to_string())?
		.to_string();

	runtime
		.Environment
		.UpdateStorageValue(true, Key, None)
		.await
		.map_err(|Error| format!("storage:delete failed: {}", Error))?;

	Ok(Value::Null)
}

/// Return all storage keys.
async fn handle_storage_keys(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	use CommonLibrary::Storage::StorageProvider::StorageProvider;

	let Storage = runtime
		.Environment
		.GetAllStorage(true)
		.await
		.map_err(|Error| format!("storage:keys failed: {}", Error))?;

	let Keys:Vec<String> = Storage.as_object().map(|O| O.keys().cloned().collect()).unwrap_or_default();
	Ok(json!(Keys))
}

// ============================================================================
// Notification handlers
// ============================================================================

/// Show a notification message — emits sky://notification/show for Sky to
/// render.
async fn handle_notification_show(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Message = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Severity = args.get(1).and_then(|V| V.as_str()).unwrap_or("info").to_string();
	let Actions = args.get(2).cloned().unwrap_or(json!([]));

	let Id = format!(
		"notification-{}",
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|D| D.as_millis())
			.unwrap_or(0)
	);

	let _ = app_handle.emit(
		"sky://notification/show",
		json!({
			"id": Id,
			"message": Message,
			"severity": Severity,
			"actions": Actions,
		}),
	);

	Ok(json!(Id))
}

/// Begin a progress notification — emits sky://notification/progress-begin.
async fn handle_notification_show_progress(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Title = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Cancellable = args.get(1).and_then(|V| V.as_bool()).unwrap_or(false);

	let Id = format!(
		"progress-{}",
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|D| D.as_millis())
			.unwrap_or(0)
	);

	let _ = app_handle.emit(
		"sky://notification/progress-begin",
		json!({
			"id": Id,
			"title": Title,
			"cancellable": Cancellable,
		}),
	);

	Ok(json!(Id))
}

/// Update an in-progress notification progress bar.
async fn handle_notification_update_progress(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Increment = args.get(1).and_then(|V| V.as_f64()).unwrap_or(0.0);
	let Message = args.get(2).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = app_handle.emit(
		"sky://notification/progress-update",
		json!({
			"id": Id,
			"increment": Increment,
			"message": Message,
		}),
	);

	Ok(Value::Null)
}

/// End a progress notification.
async fn handle_notification_end_progress(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = app_handle.emit("sky://notification/progress-end", json!({ "id": Id }));

	Ok(Value::Null)
}

// ============================================================================
// Progress handlers
// ============================================================================

/// Begin a window-level or status-bar progress indicator.
async fn handle_progress_begin(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Location = args.first().and_then(|V| V.as_str()).unwrap_or("notification").to_string();
	let Title = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Cancellable = args.get(2).and_then(|V| V.as_bool()).unwrap_or(false);

	let Id = format!(
		"progress-{}",
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|D| D.as_millis())
			.unwrap_or(0)
	);

	let _ = app_handle.emit(
		"sky://progress/begin",
		json!({
			"id": Id,
			"location": Location,
			"title": Title,
			"cancellable": Cancellable,
		}),
	);

	Ok(json!(Id))
}

/// Report incremental progress on an active indicator.
async fn handle_progress_report(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	let Increment = args.get(1).and_then(|V| V.as_f64()).unwrap_or(0.0);
	let Message = args.get(2).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = app_handle.emit(
		"sky://progress/report",
		json!({
			"id": Id,
			"increment": Increment,
			"message": Message,
		}),
	);

	Ok(Value::Null)
}

/// End a progress indicator.
async fn handle_progress_end(app_handle:tauri::AppHandle, args:Vec<Value>) -> Result<Value, String> {
	use tauri::Emitter;

	let Id = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	let _ = app_handle.emit("sky://progress/end", json!({ "id": Id }));

	Ok(Value::Null)
}

// ============================================================================
// QuickInput handlers
// ============================================================================

/// Show a quick-pick dialog. Routes through UserInterfaceProvider (blocking
/// oneshot).
async fn handle_quick_input_show_quick_pick(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::UserInterface::{
		DTO::{QuickPickItemDTO::QuickPickItemDTO, QuickPickOptionsDTO::QuickPickOptionsDTO},
		UserInterfaceProvider::UserInterfaceProvider,
	};

	let Items:Vec<QuickPickItemDTO> = args
		.first()
		.and_then(|V| V.as_array())
		.map(|Arr| {
			Arr.iter()
				.filter_map(|Item| {
					let Label = Item.get("label").and_then(|L| L.as_str()).unwrap_or("").to_string();
					let Description = Item.get("description").and_then(|D| D.as_str()).map(|S| S.to_string());
					let Detail = Item.get("detail").and_then(|D| D.as_str()).map(|S| S.to_string());
					let Picked = Item.get("picked").and_then(|P| P.as_bool()).unwrap_or(false);
					Some(QuickPickItemDTO { Label, Description, Detail, Picked:Some(Picked), AlwaysShow:Some(false) })
				})
				.collect()
		})
		.unwrap_or_default();

	let Options = QuickPickOptionsDTO {
		PlaceHolder:args
			.get(1)
			.and_then(|V| V.get("placeholder"))
			.and_then(|P| P.as_str())
			.map(|S| S.to_string()),
		CanPickMany:Some(
			args.get(1)
				.and_then(|V| V.get("canPickMany"))
				.and_then(|B| B.as_bool())
				.unwrap_or(false),
		),
		Title:args
			.get(1)
			.and_then(|V| V.get("title"))
			.and_then(|T| T.as_str())
			.map(|S| S.to_string()),
		..Default::default()
	};

	let Result = runtime
		.Environment
		.ShowQuickPick(Items, Some(Options))
		.await
		.map_err(|Error| format!("quickInput:showQuickPick failed: {}", Error))?;

	match Result {
		Some(Labels) => Ok(Labels.into_iter().next().map(|S| json!(S)).unwrap_or(Value::Null)),
		None => Ok(Value::Null),
	}
}

/// Show an input box dialog. Routes through UserInterfaceProvider (blocking
/// oneshot).
async fn handle_quick_input_show_input_box(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::UserInterface::{
		DTO::InputBoxOptionsDTO::InputBoxOptionsDTO,
		UserInterfaceProvider::UserInterfaceProvider,
	};

	let Opts = args.first();
	let Options = InputBoxOptionsDTO {
		Prompt:Opts
			.and_then(|V| V.get("prompt"))
			.and_then(|P| P.as_str())
			.map(|S| S.to_string()),
		PlaceHolder:Opts
			.and_then(|V| V.get("placeholder"))
			.and_then(|P| P.as_str())
			.map(|S| S.to_string()),
		IsPassword:Some(Opts.and_then(|V| V.get("password")).and_then(|B| B.as_bool()).unwrap_or(false)),
		Value:Opts
			.and_then(|V| V.get("value"))
			.and_then(|V| V.as_str())
			.map(|S| S.to_string()),
		Title:Opts
			.and_then(|V| V.get("title"))
			.and_then(|T| T.as_str())
			.map(|S| S.to_string()),
		IgnoreFocusOut:None,
	};

	let Result = runtime
		.Environment
		.ShowInputBox(Some(Options))
		.await
		.map_err(|Error| format!("quickInput:showInputBox failed: {}", Error))?;

	Ok(Result.map(|S| json!(S)).unwrap_or(Value::Null))
}

// ============================================================================
// Workspaces handlers
// ============================================================================

/// Return the current workspace folders.
async fn handle_workspaces_get_folders(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Workspace = &runtime.Environment.ApplicationState.Workspace;
	let Folders = Workspace.GetWorkspaceFolders();

	let FolderList:Vec<Value> = Folders
		.iter()
		.enumerate()
		.map(|(Index, Folder)| {
			json!({
				"uri": Folder.URI.to_string(),
				"name": Folder.Name,
				"index": Index,
			})
		})
		.collect();

	Ok(json!(FolderList))
}

/// Add a workspace folder.
async fn handle_workspaces_add_folder(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use url::Url;

	let UriStr = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workspaces:addFolder requires uri as first argument".to_string())?
		.to_string();

	let Name = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Workspace = &runtime.Environment.ApplicationState.Workspace;
	let mut Folders = Workspace.GetWorkspaceFolders();
	let Index = Folders.len();
	let URI = Url::parse(&UriStr).map_err(|E| format!("workspaces:addFolder invalid URI: {}", E))?;
	if let Ok(Folder) = WorkspaceFolderStateDTO::New(URI, Name, Index) {
		Folders.push(Folder);
		Workspace.SetWorkspaceFolders(Folders);
	}

	Ok(Value::Null)
}

/// Remove a workspace folder by URI.
async fn handle_workspaces_remove_folder(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let UriStr = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workspaces:removeFolder requires uri as first argument".to_string())?
		.to_string();

	let Workspace = &runtime.Environment.ApplicationState.Workspace;
	let mut Folders = Workspace.GetWorkspaceFolders();
	Folders.retain(|F| F.URI.to_string() != UriStr);
	for (I, F) in Folders.iter_mut().enumerate() {
		F.Index = I;
	}
	Workspace.SetWorkspaceFolders(Folders);

	Ok(Value::Null)
}

/// Return the workspace name (basename of root folder, or None if untitled).
async fn handle_workspaces_get_name(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Name = runtime
		.Environment
		.ApplicationState
		.Workspace
		.GetWorkspaceFolders()
		.into_iter()
		.next()
		.map(|F| F.GetDisplayName());

	Ok(Name.map(|N| json!(N)).unwrap_or(Value::Null))
}

// ============================================================================
// Themes handlers
// ============================================================================

/// Return the active color theme metadata from ConfigurationProvider.
async fn handle_themes_get_active(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	use CommonLibrary::Configuration::{
		ConfigurationProvider::ConfigurationProvider,
		DTO::ConfigurationOverridesDTO::ConfigurationOverridesDTO,
	};

	let ThemeId = runtime
		.Environment
		.GetConfigurationValue(Some("workbench.colorTheme".to_string()), ConfigurationOverridesDTO::default())
		.await
		.map_err(|Error| format!("themes:getActive failed: {}", Error))?;

	let Id = ThemeId.as_str().unwrap_or("Default Dark Modern").to_string();

	// Infer kind from id string
	let Kind = if Id.to_lowercase().contains("light") {
		"light"
	} else if Id.to_lowercase().contains("high contrast light") {
		"highContrastLight"
	} else if Id.to_lowercase().contains("high contrast") {
		"highContrast"
	} else {
		"dark"
	};

	Ok(json!({ "id": Id, "label": Id, "kind": Kind }))
}

/// Return installed theme extensions.
async fn handle_themes_list(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	// For now return a hardcoded set of built-in themes; extensions contribute
	// more.
	let Themes = vec![
		json!({ "id": "Default Dark Modern", "label": "Default Dark Modern", "kind": "dark" }),
		json!({ "id": "Default Light Modern", "label": "Default Light Modern", "kind": "light" }),
		json!({ "id": "Default Dark+", "label": "Default Dark+", "kind": "dark" }),
		json!({ "id": "Default Light+", "label": "Default Light+", "kind": "light" }),
		json!({ "id": "High Contrast", "label": "High Contrast", "kind": "highContrast" }),
		json!({ "id": "High Contrast Light", "label": "High Contrast Light", "kind": "highContrastLight" }),
	];

	Ok(json!(Themes))
}

/// Set the active color theme by updating ConfigurationProvider.
async fn handle_themes_set(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Configuration::{
		ConfigurationProvider::ConfigurationProvider,
		DTO::{ConfigurationOverridesDTO::ConfigurationOverridesDTO, ConfigurationTarget::ConfigurationTarget},
	};
	use tauri::Emitter;

	let ThemeId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("themes:set requires themeId as first argument".to_string())?
		.to_string();

	runtime
		.Environment
		.UpdateConfigurationValue(
			"workbench.colorTheme".to_string(),
			json!(ThemeId),
			ConfigurationTarget::User,
			ConfigurationOverridesDTO::default(),
			None,
		)
		.await
		.map_err(|Error| format!("themes:set failed: {}", Error))?;

	let _ = runtime
		.Environment
		.ApplicationHandle
		.emit("sky://theme/change", json!({ "themeId": ThemeId }));

	Ok(Value::Null)
}

// ============================================================================
// Search handlers
// ============================================================================

/// Search text across all workspace files (line-by-line grep, max 1000
/// results).
async fn handle_search_find_in_files(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use std::path::PathBuf;

	use globset::GlobBuilder;
	use tokio::fs;

	let Pattern = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("search:findInFiles requires pattern".to_string())?
		.to_string();
	let IsRegex = args.get(1).and_then(|V| V.as_bool()).unwrap_or(false);
	let IsCaseSensitive = args.get(2).and_then(|V| V.as_bool()).unwrap_or(false);
	let _IsWordMatch = args.get(3).and_then(|V| V.as_bool()).unwrap_or(false);
	let IncludeGlob = args.get(4).and_then(|V| V.as_str()).unwrap_or("**").to_string();
	let ExcludeGlob = args.get(5).and_then(|V| V.as_str()).unwrap_or("").to_string();
	let MaxResults = args.get(6).and_then(|V| V.as_u64()).unwrap_or(1000) as usize;

	let WorkspaceFolders = runtime.Environment.ApplicationState.Workspace.GetWorkspaceFolders();

	if WorkspaceFolders.is_empty() {
		return Ok(json!([]));
	}

	let RootPath = PathBuf::from(&WorkspaceFolders[0].URI.to_string().replace("file://", ""));

	// Build include matcher
	let IncludeMatcher = GlobBuilder::new(&IncludeGlob)
		.literal_separator(false)
		.build()
		.map(|G| G.compile_matcher())
		.ok();

	// Build exclude matcher
	let ExcludeMatcher = if !ExcludeGlob.is_empty() {
		GlobBuilder::new(&ExcludeGlob)
			.literal_separator(false)
			.build()
			.map(|G| G.compile_matcher())
			.ok()
	} else {
		None
	};

	let SearchText = Pattern.clone();
	let mut Matches = Vec::new();

	// Walk directory recursively
	let mut Stack = vec![RootPath.clone()];
	while let Some(Dir) = Stack.pop() {
		let mut Entries = match fs::read_dir(&Dir).await {
			Ok(E) => E,
			Err(_) => continue,
		};

		while let Ok(Some(Entry)) = Entries.next_entry().await {
			let Path = Entry.path();
			let RelPath = Path.strip_prefix(&RootPath).unwrap_or(&Path).to_string_lossy().to_string();

			// Skip hidden dirs
			if Path.file_name().map(|N| N.to_string_lossy().starts_with('.')).unwrap_or(false) {
				continue;
			}

			if Path.is_dir() {
				Stack.push(Path);
				continue;
			}

			// Check include/exclude globs
			if let Some(Ref) = &IncludeMatcher {
				if !Ref.is_match(&RelPath) {
					continue;
				}
			}
			if let Some(Ref) = &ExcludeMatcher {
				if Ref.is_match(&RelPath) {
					continue;
				}
			}

			// Read file and search line by line
			let Content = match fs::read_to_string(&Path).await {
				Ok(C) => C,
				Err(_) => continue,
			};

			for (LineIndex, Line) in Content.lines().enumerate() {
				let Hit = if IsRegex {
					// Simple contains fallback (no regex crate available here)
					Line.contains(&SearchText)
				} else if IsCaseSensitive {
					Line.contains(&SearchText)
				} else {
					Line.to_lowercase().contains(&SearchText.to_lowercase())
				};

				if Hit {
					let Uri = format!("file://{}", Path.to_string_lossy());
					Matches.push(json!({
						"uri": Uri,
						"lineNumber": LineIndex + 1,
						"preview": Line.trim(),
					}));

					if Matches.len() >= MaxResults {
						return Ok(json!(Matches));
					}
				}
			}
		}
	}

	Ok(json!(Matches))
}

/// Search file paths by glob pattern in workspace.
async fn handle_search_find_files(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	use std::path::PathBuf;

	use globset::GlobBuilder;
	use tokio::fs;

	let Pattern = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("search:findFiles requires pattern".to_string())?
		.to_string();
	let MaxResults = args.get(1).and_then(|V| V.as_u64()).unwrap_or(500) as usize;

	let WorkspaceFolders = runtime.Environment.ApplicationState.Workspace.GetWorkspaceFolders();

	if WorkspaceFolders.is_empty() {
		return Ok(json!([]));
	}

	let RootPath = PathBuf::from(&WorkspaceFolders[0].URI.to_string().replace("file://", ""));

	let Matcher = GlobBuilder::new(&Pattern)
		.literal_separator(false)
		.build()
		.map(|G| G.compile_matcher())
		.map_err(|Error| format!("Invalid glob pattern: {}", Error))?;

	let mut Files = Vec::new();
	let mut Stack = vec![RootPath.clone()];

	while let Some(Dir) = Stack.pop() {
		let mut Entries = match fs::read_dir(&Dir).await {
			Ok(E) => E,
			Err(_) => continue,
		};

		while let Ok(Some(Entry)) = Entries.next_entry().await {
			let Path = Entry.path();

			if Path.file_name().map(|N| N.to_string_lossy().starts_with('.')).unwrap_or(false) {
				continue;
			}

			if Path.is_dir() {
				Stack.push(Path);
				continue;
			}

			let RelPath = Path.strip_prefix(&RootPath).unwrap_or(&Path).to_string_lossy().to_string();

			if Matcher.is_match(&RelPath) {
				Files.push(format!("file://{}", Path.to_string_lossy()));

				if Files.len() >= MaxResults {
					return Ok(json!(Files));
				}
			}
		}
	}

	Ok(json!(Files))
}

// ============================================================================
// Decorations handlers
// ============================================================================

/// Return the decoration (badge, tooltip, color) for a single URI.
/// Mountain holds decorations in ApplicationState; extensions push them via
/// the `decorations:set` IPC channel.
async fn handle_decorations_get(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("decorations:get requires uri".to_string())?;
	let Decoration = runtime.Environment.ApplicationState.Feature.Decorations.GetDecoration(Uri);
	Ok(Decoration.unwrap_or(Value::Null))
}

/// Return decorations for multiple URIs in a single round-trip.
async fn handle_decorations_get_many(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uris:Vec<String> = args
		.first()
		.and_then(|V| V.as_array())
		.map(|Arr| Arr.iter().filter_map(|U| U.as_str().map(str::to_owned)).collect())
		.unwrap_or_default();

	let mut Result = serde_json::Map::new();
	for Uri in &Uris {
		if let Some(Decoration) = runtime.Environment.ApplicationState.Feature.Decorations.GetDecoration(Uri) {
			Result.insert(Uri.clone(), Decoration);
		}
	}
	Ok(Value::Object(Result))
}

/// Register or override the decoration for a URI.
async fn handle_decorations_set(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("decorations:set requires uri".to_string())?;
	let Decoration = args.get(1).cloned().unwrap_or(Value::Null);
	runtime
		.Environment
		.ApplicationState
		.Feature
		.Decorations
		.SetDecoration(Uri, Decoration);
	Ok(Value::Null)
}

/// Remove the decoration for a URI.
async fn handle_decorations_clear(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("decorations:clear requires uri".to_string())?;
	runtime.Environment.ApplicationState.Feature.Decorations.ClearDecoration(Uri);
	Ok(Value::Null)
}

// ============================================================================
// WorkingCopy handlers
// ============================================================================

/// Check whether a URI has unsaved changes.
async fn handle_working_copy_is_dirty(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workingCopy:isDirty requires uri".to_string())?;
	let IsDirty = runtime.Environment.ApplicationState.Feature.WorkingCopy.IsDirty(Uri);
	Ok(json!(IsDirty))
}

/// Mark a URI as dirty (unsaved) or clean.
async fn handle_working_copy_set_dirty(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workingCopy:setDirty requires uri".to_string())?;
	let Dirty = args.get(1).and_then(|V| V.as_bool()).unwrap_or(true);
	runtime.Environment.ApplicationState.Feature.WorkingCopy.SetDirty(Uri, Dirty);
	Ok(Value::Null)
}

/// Return all URIs that currently have unsaved changes.
async fn handle_working_copy_get_all_dirty(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Dirty = runtime.Environment.ApplicationState.Feature.WorkingCopy.GetAllDirty();
	Ok(json!(Dirty))
}

/// Return the count of resources with unsaved changes.
async fn handle_working_copy_get_dirty_count(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Count = runtime.Environment.ApplicationState.Feature.WorkingCopy.GetDirtyCount();
	Ok(json!(Count))
}

// ============================================================================
// Keybinding handlers
// ============================================================================

/// Register a dynamic keybinding in Mountain's keybinding registry.
async fn handle_keybinding_add(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let CommandId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:add requires commandId".to_string())?
		.to_owned();
	let KeyExpression = args
		.get(1)
		.and_then(|V| V.as_str())
		.ok_or("keybinding:add requires keybinding".to_string())?
		.to_owned();
	let When = args.get(2).and_then(|V| V.as_str()).map(str::to_owned);
	runtime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.AddKeybinding(CommandId, KeyExpression, When);
	Ok(Value::Null)
}

/// Remove all dynamic keybindings for a command.
async fn handle_keybinding_remove(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let CommandId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:remove requires commandId".to_string())?;
	runtime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.RemoveKeybinding(CommandId);
	Ok(Value::Null)
}

/// Look up the keybinding string for a command.
async fn handle_keybinding_lookup(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let CommandId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:lookup requires commandId".to_string())?;
	let Binding = runtime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.LookupKeybinding(CommandId);
	Ok(Binding.map(|B| json!(B)).unwrap_or(Value::Null))
}

/// Return all registered dynamic keybindings.
async fn handle_keybinding_get_all(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let All = runtime.Environment.ApplicationState.Feature.Keybindings.GetAllKeybindings();
	Ok(json!(All))
}

// ============================================================================
// Lifecycle handlers
// ============================================================================

/// Return the current application lifecycle phase (1–4).
async fn handle_lifecycle_get_phase(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Phase = runtime.Environment.ApplicationState.Feature.Lifecycle.GetPhase();
	Ok(json!(Phase))
}

/// Wait (poll) until the application reaches at least the requested phase.
/// Returns immediately if the phase has already been reached.
async fn handle_lifecycle_when_phase(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let RequestedPhase = args.first().and_then(|V| V.as_u64()).unwrap_or(1) as u8;
	let CurrentPhase = runtime.Environment.ApplicationState.Feature.Lifecycle.GetPhase();
	if CurrentPhase >= RequestedPhase {
		return Ok(Value::Null);
	}
	// Simple poll with short sleep — production should use a channel/notify
	let mut Retries = 0u8;
	loop {
		tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
		let Phase = runtime.Environment.ApplicationState.Feature.Lifecycle.GetPhase();
		if Phase >= RequestedPhase || Retries >= 50 {
			break;
		}
		Retries += 1;
	}
	Ok(Value::Null)
}

/// Initiate a graceful application shutdown via Tauri.
async fn handle_lifecycle_request_shutdown(app_handle:AppHandle) -> Result<Value, String> {
	app_handle.exit(0);
	Ok(Value::Null)
}
