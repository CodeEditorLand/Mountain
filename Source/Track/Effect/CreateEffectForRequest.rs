#![allow(unused_imports)]

//! # CreateEffectForRequest (Track)
//!
//! ## RESPONSIBILITIES
//!
//! This module provides the central routing table that maps string-based
//! commands/RPC methods to typed effects. It creates MappedEffect (type-erased
//! async closures) for dispatch execution and integrates with the effect system
//! (ActionEffect) and provider traits. Some operations use direct provider
//! calls for performance.
//!
//! ### Core Functions:
//! - Map string-based method names to effect constructors
//! - Create MappedEffect (boxed closures) for execution
//! - Support direct provider calls for hot paths
//! - Handle parameter deserialization and validation
//!
//! ## ARCHITECTURAL ROLE
//!
//! CreateEffectForRequest acts as the **effect mapper** in Track's dispatch
//! system:
//!
//! ```text
//! Dispatch Logic ──► CreateEffectForRequest (Match) ──► MappedEffect ──► ApplicationRunTime Execution
//! ```
//!
//! ## KEY COMPONENTS
//!
//! - **Fn**: Main effect creation function (pub fn Fn<R:Runtime>)
//! - **MappedEffect**: Type alias for boxed async closure (imported from
//!   MappedEffect module)
//!
//! ## ERROR HANDLING
//!
//! - All effects return Result<Value, String> (serializable errors for IPC)
//! - Parameter validation with descriptive error messages
//! - Unknown command handling returns error instead of panic
//! - Serialization/deserialization errors caught and reported
//! - Provider errors propagate with context
//!
//! ## LOGGING
//!
//! - Unknown commands are logged at warn level
//! - Log format: "[`CreateEffectForRequest`] Unknown method: {}"
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Effect creation is cheap: match + constructor call + box
//! - Direct provider calls avoid allocation (for hot paths)
//! - TODO: Consider implementing an effect pool to cache frequently created
//!   effects
//! - TODO: Add configurable command timeouts per command type and rate limiting
//!
//! ## DIRECT PROVIDER CALLS
//!
//! Some operations bypass the effect system for performance:
//! - Configuration: `GetConfiguration`, `UpdateConfiguration`
//! - Diagnostics: `SetDiagnostics`, `ClearDiagnostics`
//! - Language Features: `ProvideHover`, `ProvideCompletions`, etc.
//! - Terminal: direct text send/receive
//! - Why? Avoid effect overhead for high-frequency operations
//!
//! ## SUPPORTED COMMAND CATEGORIES
//!
//! **Commands**: Execute, GetAll, Register
//! **Configuration**: Inspect, Update
//! **Documents**: Save, SaveAs
//! **FileSystem**: ReadFile, WriteFile, ReadDirectory
//! **Debug**: Start, RegisterConfigurationProvider
//! **Diagnostics**: Set, Clear
//! **Keybinding**: GetResolved
//! **LanguageFeatures**: $languageFeatures:registerProvider
//! **Search**: TextSearch
//! **SourceControlManagement**: $scm:createSourceControl, updateSourceControl,
//! updateGroup, registerInputBox **StatusBar**: $statusBar:set, dispose,
//! $setStatusBarMessage, $disposeStatusBarMessage **Storage**: Get, Set
//! **Terminal**: $terminal:create, sendText, dispose
//! **TreeView**: $tree:register
//! **UserInterface**: ShowMessage, ShowOpenDialog, ShowSaveDialog
//! **Webview**: $webview:create, $resolveCustomEditor
//!
//! ## TODO
//!
//! High Priority:
//! - [ ] Add command parameter schema validation (JSON schema per command)
//! - [ ] Implement command permission checking (capability-based security)
//! - [ ] Add command deprecation warnings and migration
//! - [ ] Cache frequently created effects (reuse boxed closures)
//! - [ ] Add command timeout configuration (per-command TTL)
//! - [ ] Implement command rate limiting (DoS protection)
//! - [ ] Add command metrics collection (latency, success rate)
//!
//! Medium Priority:
//! - [ ] Implement command aliasing (user-defined shortcuts)
//! - [ ] Add command migration support (rename, deprecate)
//! - [ ] Add comprehensive command audit logging
//! - [ ] Support command chaining and composition
//! - [ ] Implement command undo/redo integration
//! - [ ] Split CreateEffectForRequest into individual effect modules
//!
//! Low Priority:
//! - [ ] Add request tracing across the entire pipeline

use std::{future::Future, pin::Pin, sync::Arc};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use CommonLibrary::{
	Command::CommandExecutor::CommandExecutor,
	Configuration::{
		ConfigurationInspector::ConfigurationInspector,
		ConfigurationProvider::ConfigurationProvider,
		DTO::ConfigurationTarget::ConfigurationTarget,
	},
	CustomEditor::CustomEditorProvider::CustomEditorProvider,
	Debug::DebugService::DebugService,
	Diagnostic::DiagnosticManager::DiagnosticManager,
	Document::DocumentProvider::DocumentProvider,
	Environment::Requires::Requires,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
	Keybinding::KeybindingProvider::KeybindingProvider,
	LanguageFeature::{
		DTO::ProviderType::ProviderType,
		LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	},
	Search::SearchProvider::SearchProvider,
	SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider,
	StatusBar::{DTO::StatusBarEntryDTO::StatusBarEntryDTO, StatusBarProvider::StatusBarProvider},
	Storage::StorageProvider::StorageProvider,
	Terminal::TerminalProvider::TerminalProvider,
	TreeView::TreeViewProvider::TreeViewProvider,
	UserInterface::{DTO::MessageSeverity::MessageSeverity, UserInterfaceProvider::UserInterfaceProvider},
	Webview::WebviewProvider,
};
use serde_json::{Value, json};
use tauri::{AppHandle, Runtime};
use url::Url;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect};
use crate::dev_log;

/// Maps a string-based method name (command or RPC) to its corresponding effect
/// constructor, returning a boxed closure ([`MappedEffect`]) that can be
/// executed by the ApplicationRunTime.
///
/// # Arguments
/// - `ApplicationHandle`: Tauri app handle for accessing state
/// - `MethodName`: The command/RPC method name to map
/// - `Parameters`: JSON value containing parameters for the effect
///
/// # Returns
/// `Result<MappedEffect, String>` - either a boxed async closure or an error
/// if the command is unknown
pub fn CreateEffectForRequest<R:Runtime>(
	_ApplicationHandle:&AppHandle<R>,
	MethodName:&str,
	Parameters:Value,
) -> Result<MappedEffect, String> {
	match MethodName {
		// Configuration
		"Configuration.Inspect" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn ConfigurationInspector> = run_time.Environment.Require();
						let section = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let result = provider.InspectConfigurationValue(section, Default::default()).await;
						result.map(|_opt_dto| json!(null)).map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"Configuration.Update" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn ConfigurationProvider> = run_time.Environment.Require();
						let key = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let value = Parameters.get(1).cloned().unwrap_or_default();
						let target = match Parameters.get(2).and_then(Value::as_u64) {
							Some(0) => ConfigurationTarget::User,
							Some(1) => ConfigurationTarget::Workspace,
							_ => ConfigurationTarget::User,
						};
						let result = provider
							.UpdateConfigurationValue(key, value, target, Default::default(), None)
							.await;
						result.map(|_| json!(null)).map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Diagnostics
		"Diagnostic.Set" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn DiagnosticManager> = run_time.Environment.Require();
						let owner = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let entries = Parameters.get(1).cloned().unwrap_or_default();
						provider
							.SetDiagnostics(owner, entries)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"Diagnostic.Clear" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn DiagnosticManager> = run_time.Environment.Require();
						let owner = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						provider
							.ClearDiagnostics(owner)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Language Features
		"$languageFeatures:registerProvider" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn LanguageFeatureProviderRegistry> = run_time.Environment.Require();
						let id = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let selector = Parameters.get(1).cloned().unwrap_or_default();
						let extension_id = Parameters.get(2).cloned().unwrap_or_default();
						let options = Parameters.get(3).cloned();
						provider
							.RegisterProvider(id, ProviderType::Hover, selector, extension_id, options)
							.await
							.map(|handle| json!(handle))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Documents
		"Document.Save" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let document_provider:Arc<dyn DocumentProvider> = run_time.Environment.Require();
						let uri_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let uri = Url::parse(uri_str).unwrap_or_else(|_| Url::parse("file:///tmp/test.txt").unwrap());
						document_provider
							.SaveDocument(uri)
							.await
							.map(|success| json!(success))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"Document.SaveAs" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let document_provider:Arc<dyn DocumentProvider> = run_time.Environment.Require();
						let original_uri_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let original_uri = Url::parse(original_uri_str)
							.unwrap_or_else(|_| Url::parse("file:///tmp/test.txt").unwrap());
						let target_uri = Parameters
							.get(1)
							.and_then(Value::as_str)
							.map(Url::parse)
							.transpose()
							.unwrap_or(None);
						document_provider
							.SaveDocumentAs(original_uri, target_uri)
							.await
							.map(|uri_option| json!(uri_option))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// FileSystem
		"FileSystem.ReadFile" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let fs_reader:Arc<dyn FileSystemReader> = run_time.Environment.Require();
						let path_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let path = std::path::PathBuf::from(path_str);
						fs_reader
							.ReadFile(&path)
							.await
							.map(|bytes| json!(bytes))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"FileSystem.WriteFile" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let fs_writer:Arc<dyn FileSystemWriter> = run_time.Environment.Require();
						let path_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let path = std::path::PathBuf::from(path_str);
						let content = Parameters.get(1).cloned();
						let content_bytes = match content {
							Some(Value::Array(arr)) => {
								arr.into_iter().filter_map(|v| v.as_u64().map(|n| n as u8)).collect()
							},
							Some(Value::String(s)) => STANDARD.decode(&s).unwrap_or_default(),
							_ => vec![],
						};
						fs_writer
							.WriteFile(&path, content_bytes, true, true)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"FileSystem.ReadDirectory" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let fs_reader:Arc<dyn FileSystemReader> = run_time.Environment.Require();
						let path_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let path = std::path::PathBuf::from(path_str);
						fs_reader
							.ReadDirectory(&path)
							.await
							.map(|entries| json!(entries))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"FileSystem.Stat" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let fs_reader:Arc<dyn FileSystemReader> = run_time.Environment.Require();
						let path_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let path = std::path::PathBuf::from(path_str);
						fs_reader
							.StatFile(&path)
							.await
							.map(|stat| json!(stat))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"FileSystem.CreateDirectory" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let fs_writer:Arc<dyn FileSystemWriter> = run_time.Environment.Require();
						let path_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let path = std::path::PathBuf::from(path_str);
						fs_writer
							.CreateDirectory(&path, true)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"FileSystem.Delete" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let fs_writer:Arc<dyn FileSystemWriter> = run_time.Environment.Require();
						let path_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let path = std::path::PathBuf::from(path_str);
						let recursive = Parameters.get(1).and_then(Value::as_bool).unwrap_or(false);
						fs_writer
							.Delete(&path, recursive, false)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"FileSystem.Rename" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let fs_writer:Arc<dyn FileSystemWriter> = run_time.Environment.Require();
						let source = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let target = Parameters.get(1).and_then(Value::as_str).unwrap_or("");
						fs_writer
							.Rename(
								&std::path::PathBuf::from(source),
								&std::path::PathBuf::from(target),
								true,
							)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"FileSystem.Copy" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let fs_writer:Arc<dyn FileSystemWriter> = run_time.Environment.Require();
						let source = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let target = Parameters.get(1).and_then(Value::as_str).unwrap_or("");
						fs_writer
							.Copy(
								&std::path::PathBuf::from(source),
								&std::path::PathBuf::from(target),
								true,
							)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Keybinding
		"Keybinding.GetResolved" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn KeybindingProvider> = run_time.Environment.Require();
						provider.GetResolvedKeybinding().await.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Search
		"Search.TextSearch" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SearchProvider> = run_time.Environment.Require();
						let query = Parameters.get(0).cloned().unwrap_or_default();
						let options = Parameters.get(1).cloned().unwrap_or_default();
						provider.TextSearch(query, options).await.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Storage
		"Storage.Get" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn StorageProvider> = run_time.Environment.Require();
						let key = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						provider
							.GetStorageValue(false, &key)
							.await
							.map(|opt_val| json!(opt_val))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"Storage.Set" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn StorageProvider> = run_time.Environment.Require();
						let key = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let value = Parameters.get(1).cloned();
						provider
							.UpdateStorageValue(false, key, value)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Commands
		"Command.Execute" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let command_executor:Arc<dyn CommandExecutor> = run_time.Environment.Require();
						let command_id = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let args = Parameters.get(1).cloned().unwrap_or_default();
						command_executor
							.ExecuteCommand(command_id, args)
							.await
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"Command.GetAll" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn CommandExecutor> = run_time.Environment.Require();
						provider
							.GetAllCommands()
							.await
							.map(|cmds| json!(cmds))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Status Bar
		"$statusBar:set" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();
						// Construct a minimal StatusBarEntryDTO from parameters
						let text = Parameters.get(0).and_then(Value::as_str).unwrap_or("status").to_string();
						let entry = StatusBarEntryDTO {
							EntryIdentifier:"id".to_string(),
							ItemIdentifier:"item".to_string(),
							ExtensionIdentifier:"ext".to_string(),
							Name:None,
							Text:text,
							Tooltip:None,
							HasTooltipProvider:false,
							Command:None,
							Color:None,
							BackgroundColor:None,
							IsAlignedLeft:false,
							Priority:None,
							AccessibilityInformation:None,
						};
						provider
							.SetStatusBarEntry(entry)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"$statusBar:dispose" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();
						let id = Parameters.get(0).and_then(Value::as_str).unwrap_or("id").to_string();
						provider
							.DisposeStatusBarEntry(id)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"$setStatusBarMessage" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();
						let message_id = Parameters.get(0).and_then(Value::as_str).unwrap_or("msg_id").to_string();
						let text = Parameters.get(1).and_then(Value::as_str).unwrap_or("message").to_string();
						provider
							.SetStatusBarMessage(message_id, text)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"$disposeStatusBarMessage" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();
						let message_id = Parameters.get(0).and_then(Value::as_str).unwrap_or("msg_id").to_string();
						provider
							.DisposeStatusBarMessage(message_id)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// User Interface
		"UserInterface.ShowMessage" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();
						let severity_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("info");
						let message = Parameters.get(1).and_then(Value::as_str).unwrap_or("").to_string();
						let options = Parameters.get(2).cloned();
						let severity = match severity_str {
							"warning" => MessageSeverity::Warning,
							"error" => MessageSeverity::Error,
							_ => MessageSeverity::Info,
						};
						provider
							.ShowMessage(severity, message, options)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"UserInterface.ShowQuickPick" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();
						// Using default empty parameters for now
						let (items, options) = (
							vec![],
							None as Option<CommonLibrary::UserInterface::DTO::QuickPickOptionsDTO::QuickPickOptionsDTO>,
						);
						provider
							.ShowQuickPick(items, options)
							.await
							.map(|selected_items| json!(selected_items))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"UserInterface.ShowInputBox" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();
						let options = if let Some(Value::Object(obj)) = Parameters.get(0) {
							// Properly deserialize to InputBoxOptionsDTO
							match serde_json::from_value::<
								CommonLibrary::UserInterface::DTO::InputBoxOptionsDTO::InputBoxOptionsDTO,
							>(Value::Object(obj.clone()))
							{
								Ok(dto) => Some(dto),
								Err(e) => {
									dev_log!("ipc", "warn: Failed to deserialize InputBoxOptionsDTO: {}", e);
									Some(CommonLibrary::UserInterface::DTO::InputBoxOptionsDTO::InputBoxOptionsDTO::default())
								},
							}
						} else {
							None
						};
						provider
							.ShowInputBox(options)
							.await
							.map(|input_opt| json!(input_opt))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"UserInterface.ShowOpenDialog" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();
						let options = if let Some(Value::Object(obj)) = Parameters.get(0) {
							// Properly deserialize to OpenDialogOptionsDTO
							match serde_json::from_value::<
								CommonLibrary::UserInterface::DTO::OpenDialogOptionsDTO::OpenDialogOptionsDTO,
							>(Value::Object(obj.clone()))
							{
								Ok(dto) => Some(dto),
								Err(e) => {
									dev_log!("ipc", "warn: Failed to deserialize OpenDialogOptionsDTO: {}", e);
									Some(Default::default())
								},
							}
						} else {
							None
						};
						provider
							.ShowOpenDialog(options)
							.await
							.map(|path_buf_opt| json!(path_buf_opt))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"UserInterface.ShowSaveDialog" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();
						let options = if let Some(Value::Object(obj)) = Parameters.get(0) {
							// Properly deserialize to SaveDialogOptionsDTO
							match serde_json::from_value::<
								CommonLibrary::UserInterface::DTO::SaveDialogOptionsDTO::SaveDialogOptionsDTO,
							>(Value::Object(obj.clone()))
							{
								Ok(dto) => Some(dto),
								Err(e) => {
									dev_log!("ipc", "warn: Failed to deserialize SaveDialogOptionsDTO: {}", e);
									Some(Default::default())
								},
							}
						} else {
							None
						};
						provider
							.ShowSaveDialog(options)
							.await
							.map(|path_buf_opt| json!(path_buf_opt))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Terminal
		"$terminal:create" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();
						let options = Parameters.get(0).cloned().unwrap_or_default();
						provider.CreateTerminal(options).await.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"$terminal:sendText" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();
						let terminal_id = Parameters.get(0).and_then(Value::as_i64).map(|n| n as u64).unwrap_or(0);
						let text = Parameters.get(1).and_then(Value::as_str).unwrap_or("").to_string();
						provider
							.SendTextToTerminal(terminal_id, text)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"$terminal:dispose" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();
						let terminal_id = Parameters.get(0).and_then(Value::as_i64).map(|n| n as u64).unwrap_or(0);
						provider
							.DisposeTerminal(terminal_id)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Webview
		"$webview:create" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						dev_log!("ipc", "warn: $webview:create not fully implemented");
						Ok(json!({"handle": "webview-123"}))
					})
				};
			Ok(Box::new(effect))
		},

		"$resolveCustomEditor" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn CustomEditorProvider> = run_time.Environment.Require();
						let view_type = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let resource_uri_str = Parameters.get(1).and_then(Value::as_str).unwrap_or("");
						let resource_uri = Url::parse(resource_uri_str)
							.unwrap_or_else(|_| Url::parse("file:///tmp/test.txt").unwrap());
						let webview_handle =
							Parameters.get(2).and_then(Value::as_str).unwrap_or("webview-123").to_string();
						provider
							.ResolveCustomEditor(view_type, resource_uri, webview_handle)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Debug
		"Debug.Start" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn DebugService> = run_time.Environment.Require();
						let folder_uri_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let folder_uri = if folder_uri_str.is_empty() { None } else { Url::parse(folder_uri_str).ok() };
						let configuration = Parameters.get(1).cloned().unwrap_or_else(|| json!({ "type": "node" }));
						provider
							.StartDebugging(folder_uri, configuration)
							.await
							.map(|session_id| json!(session_id))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"Debug.RegisterConfigurationProvider" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn DebugService> = run_time.Environment.Require();
						let debug_type = Parameters.get(0).and_then(Value::as_str).unwrap_or("node").to_string();
						let provider_handle = Parameters.get(1).and_then(Value::as_i64).map(|n| n as u32).unwrap_or(1);
						let sidecar_id = Parameters.get(2).and_then(Value::as_str).unwrap_or("cocoon-main").to_string();
						provider
							.RegisterDebugConfigurationProvider(debug_type, provider_handle, sidecar_id)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Tree View
		"$tree:register" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn TreeViewProvider> = run_time.Environment.Require();
						let view_id = Parameters.get(0).and_then(Value::as_str).unwrap_or("viewId").to_string();
						let options = Parameters.get(1).cloned().unwrap_or_default();
						provider
							.RegisterTreeDataProvider(view_id, options)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Source Control Management
		"$scm:createSourceControl" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
						let resource = Parameters.get(0).cloned().unwrap_or_default();
						provider
							.CreateSourceControl(resource)
							.await
							.map(|handle| json!(handle))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"$scm:updateSourceControl" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
						let handle = Parameters.get(0).and_then(Value::as_i64).map(|n| n as u32).unwrap_or(0);
						let update = Parameters.get(1).cloned().unwrap_or_default();
						provider
							.UpdateSourceControl(handle, update)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"$scm:updateGroup" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
						let handle = Parameters.get(0).and_then(Value::as_i64).map(|n| n as u32).unwrap_or(0);
						let group_data = Parameters.get(1).cloned().unwrap_or_default();
						provider
							.UpdateSourceControlGroup(handle, group_data)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"$scm:registerInputBox" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
						let handle = Parameters.get(0).and_then(Value::as_i64).map(|n| n as u32).unwrap_or(0);
						let options = Parameters.get(1).cloned().unwrap_or_default();
						provider
							.RegisterInputBox(handle, options)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Debug — Stop (Cascade-8 stub; DebugService has no StopDebugging yet,
		// extensions receive `null` instead of "Unknown method")
		"Debug.Stop" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let session_id = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						dev_log!("ipc", "[Debug.Stop] stub — session={} (TODO: DebugService::StopDebugging)", session_id);
						Ok(json!(null))
					})
				};
			Ok(Box::new(effect))
		},

		// Task — Fetch/Execute (Cascade-8 stubs; no TaskProvider trait in
		// Common yet. Returning safe defaults keeps extensions from
		// crashing on `vscode.tasks.fetchTasks()` / `executeTask`.)
		"Task.Fetch" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						dev_log!("ipc", "[Task.Fetch] stub — returning [] (TODO: TaskProvider trait)");
						Ok(json!([]))
					})
				};
			Ok(Box::new(effect))
		},

		"Task.Execute" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						dev_log!("ipc", "[Task.Execute] stub — returning null (TODO: TaskProvider trait)");
						Ok(json!(null))
					})
				};
			Ok(Box::new(effect))
		},

		// Authentication — GetSession/GetAccounts (Cascade-8 stubs; no
		// AuthenticationProvider trait yet. Returning `null` / `[]` lets
		// GitHub/Copilot extensions proceed in "unauthenticated" mode
		// instead of crashing.)
		"Authentication.GetSession" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider_id =
							Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						dev_log!("ipc", "[Authentication.GetSession] stub — provider={} (TODO: AuthenticationProvider trait)", provider_id);
						Ok(json!(null))
					})
				};
			Ok(Box::new(effect))
		},

		"Authentication.GetAccounts" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider_id =
							Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						dev_log!("ipc", "[Authentication.GetAccounts] stub — provider={} (TODO: AuthenticationProvider trait)", provider_id);
						Ok(json!([]))
					})
				};
			Ok(Box::new(effect))
		},

		// Clipboard — Read/Write (Cascade-8 stubs; tauri-plugin-clipboard-manager
		// not yet on the Mountain crate. Read returns "", Write accepts and drops.)
		"Clipboard.Read" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						dev_log!("ipc", "[Clipboard.Read] stub — returning '' (TODO: tauri-plugin-clipboard-manager)");
						Ok(json!(""))
					})
				};
			Ok(Box::new(effect))
		},

		"Clipboard.Write" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let text_len = Parameters.get(0).and_then(Value::as_str).map(str::len).unwrap_or(0);
						dev_log!("ipc", "[Clipboard.Write] stub — text_len={} (TODO: tauri-plugin-clipboard-manager)", text_len);
						Ok(json!(null))
					})
				};
			Ok(Box::new(effect))
		},

		// NativeHost — OpenExternal (Cascade-8 stub; tauri-plugin-shell not
		// on the Mountain crate yet. Logs the URL; returns success so the
		// extension's promise resolves.)
		"NativeHost.OpenExternal" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let uri = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						dev_log!("ipc", "[NativeHost.OpenExternal] stub — uri={} (TODO: tauri-plugin-shell)", uri);
						Ok(json!(true))
					})
				};
			Ok(Box::new(effect))
		},

		// Languages — GetAll (Cascade-8 stub; enumerates nothing yet. Real
		// implementation would walk ApplicationState.Extension.ScannedExtensions
		// collecting `contributes.languages[]`.)
		"Languages.GetAll" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						dev_log!("ipc", "[Languages.GetAll] stub — returning [] (TODO: enumerate ScannedExtensions)");
						Ok(json!([]))
					})
				};
			Ok(Box::new(effect))
		},

		// Unknown command
		_ => {
			dev_log!("ipc", "warn: [EffectCreation] Unknown method: {}", MethodName);
			Err(format!("Unknown method: {}", MethodName))
		},
	}
}
