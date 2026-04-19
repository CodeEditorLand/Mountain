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
	FileSystem::{
		FileSystemReader::FileSystemReader,
		FileSystemWriter::FileSystemWriter,
		FileWatcherProvider::FileWatcherProvider,
	},
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider as IPCProviderTrait},
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
	Webview::WebviewProvider::WebviewProvider,
};
use serde_json::{Value, json};
use tauri::{AppHandle, Runtime};
use url::Url;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect, dev_log};

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
						let path_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");

						// Virtual `vscode://` resources: VS Code's language-features
						// code fetches a schemas-associations document at startup to
						// discover schema↔pattern mappings. We don't ship one, but an
						// empty well-formed payload satisfies the contract and keeps
						// the request off the 404 path.
						if path_str.starts_with("vscode://schemas-associations/") {
							let payload = serde_json::to_vec(&json!({ "schemas": [] }))
								.unwrap_or_else(|_| b"{\"schemas\":[]}".to_vec());
							return Ok(json!(payload));
						}

						let fs_reader:Arc<dyn FileSystemReader> = run_time.Environment.Require();
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
							.Rename(&std::path::PathBuf::from(source), &std::path::PathBuf::from(target), true)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"FileWatcher.Register" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let handle = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						if handle.is_empty() {
							return Err("FileWatcher.Register: empty handle".to_string());
						}
						let root_str = Parameters.get(1).and_then(Value::as_str).unwrap_or("");
						if root_str.is_empty() {
							return Err("FileWatcher.Register: empty root path".to_string());
						}
						let recursive = Parameters.get(2).and_then(Value::as_bool).unwrap_or(true);
						// Cocoon sends the compiled glob pattern as the 4th arg when
						// TierFileWatcher=Layer4 is active. Older callers pass only
						// three args — Mountain falls through to "no filter".
						let pattern = Parameters
							.get(3)
							.and_then(Value::as_str)
							.filter(|p| !p.is_empty())
							.map(str::to_string);
						let root = std::path::PathBuf::from(root_str);
						let watcher:Arc<dyn FileWatcherProvider> = run_time.Environment.Require();
						watcher
							.RegisterWatcher(handle, root, recursive, pattern)
							.await
							.map(|()| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"FileWatcher.Unregister" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let handle = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let watcher:Arc<dyn FileWatcherProvider> = run_time.Environment.Require();
						watcher
							.UnregisterWatcher(handle)
							.await
							.map(|()| json!(null))
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
							.Copy(&std::path::PathBuf::from(source), &std::path::PathBuf::from(target), true)
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

		// Terminal.Resize propagates a new cols×rows to the PTY master. The
		// shell inside receives SIGWINCH and line-editing utilities redraw.
		// Extensions call this from `vscode.window.Terminal.resize(cols,rows)`.
		//
		// Cocoon sends the handle either as a numeric id or as a string like
		// "terminal:7"; accept both shapes to stay compatible with both the
		// current Cocoon wiring and any future migration to string handles.
		"Terminal.Resize" | "$terminal:resize" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();
						let terminal_id = match Parameters.get(0) {
							Some(Value::Number(n)) => n.as_u64().unwrap_or(0),
							Some(Value::String(s)) => {
								s.rsplit(':').next().and_then(|token| token.parse::<u64>().ok()).unwrap_or(0)
							},
							_ => 0,
						};
						let cols = Parameters.get(1).and_then(Value::as_u64).map(|n| n as u16).unwrap_or(80);
						let rows = Parameters.get(2).and_then(Value::as_u64).map(|n| n as u16).unwrap_or(24);
						provider
							.ResizeTerminal(terminal_id, cols, rows)
							.await
							.map(|()| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Webview — Cocoon registers panels, registers view providers, and
		// forwards state mutations. Mountain re-emits every event as a
		// `sky://webview/<suffix>` Tauri event so the UI layer can render
		// and relay DOM → extension messages without a bespoke gRPC channel
		// per action.
		//
		// The full payload (handle + method args) is forwarded verbatim, which
		// matches the pattern established by TerminalProvider::ShowTerminal.
		// Sky's Workbench webview dispatcher subscribes once and routes per
		// method to the relevant panel/view provider.
		"$webview:create"
		| "webview.create"
		| "webview.setHtml"
		| "webview.setOptions"
		| "webview.postMessage"
		| "webview.reveal"
		| "webview.dispose"
		| "webview.registerView"
		| "webview.unregisterView"
		| "webview.registerCustomEditor"
		| "webview.unregisterCustomEditor" => {
			let Method = MethodName.to_string();
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					let Method = Method.clone();
					Box::pin(async move {
						use tauri::Emitter;
						let Handle = Parameters.get(0).cloned().unwrap_or(Value::Null);
						let Payload = json!({
							"method": Method,
							"handle": Handle,
							"args": Parameters,
						});
						// Convert `$webview:create` / `webview.create` → `create` so
						// Sky subscribes to a single clean event namespace.
						let Suffix = Method.trim_start_matches("$webview:").trim_start_matches("webview.");
						let EventName = format!("sky://webview/{}", Suffix);
						if let Err(Error) = run_time.Environment.ApplicationHandle.emit(&EventName, &Payload) {
							dev_log!("ipc", "warn: [WebviewEffect] emit {} failed: {}", EventName, Error);
						}
						Ok(json!(null))
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

		// Tree View — Cocoon registers a TreeDataProvider by handle. Mountain
		// stores the (handle, viewId) mapping so Sky's sidebar can render the
		// tree by round-tripping `tree.getChildren` back through Cocoon.
		"$tree:register" | "tree.register" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn TreeViewProvider> = run_time.Environment.Require();
						// Cocoon calls with [handle, viewId, options]; the old
						// call shape was [viewId, options]. Accept both.
						let first = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let (view_id, options) = if Parameters.get(2).is_some() {
							let vid = Parameters.get(1).and_then(Value::as_str).unwrap_or(first).to_string();
							let opts = Parameters.get(2).cloned().unwrap_or_default();
							(vid, opts)
						} else {
							let vid = first.to_string();
							let opts = Parameters.get(1).cloned().unwrap_or_default();
							(vid, opts)
						};
						provider
							.RegisterTreeDataProvider(view_id, options)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"tree.unregister" | "tree.dispose" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let handle = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						dev_log!("ipc", "[tree.unregister] handle={}", handle);
						Ok(json!(null))
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

		// Debug — Stop
		"Debug.Stop" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let session_id = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let debug_service:Arc<dyn DebugService> = run_time.Environment.Require();
						debug_service
							.StopDebugging(session_id)
							.await
							.map(|()| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Task — Fetch/Execute
		//
		// Cocoon hosts the actual TaskProvider implementations contributed by
		// extensions. Mountain forwards the call through the IPCProvider
		// reverse-RPC channel (ExtHostTaskService) and returns whatever the
		// extension gives us. If no sidecar answers within the timeout, return
		// safe defaults so `vscode.tasks.fetchTasks()` doesn't reject.
		"Task.Fetch" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let filter = Parameters.get(0).cloned().unwrap_or(Value::Null);
						let IPCProvider:Arc<dyn IPCProviderTrait> = run_time.Environment.Require();
						let Method = format!("{}$fetchTasks", ProxyTarget::ExtHostTaskService.GetTargetPrefix());
						match IPCProvider
							.SendRequestToSideCar("cocoon-main".to_string(), Method, json!([filter]), 5000)
							.await
						{
							Ok(value) => Ok(value),
							Err(error) => {
								dev_log!(
									"ipc",
									"warn: [Task.Fetch] extension did not answer ({:?}); returning []",
									error
								);
								Ok(json!([]))
							},
						}
					})
				};
			Ok(Box::new(effect))
		},

		"Task.Execute" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let task = Parameters.get(0).cloned().unwrap_or(Value::Null);
						let IPCProvider:Arc<dyn IPCProviderTrait> = run_time.Environment.Require();
						let Method = format!("{}$executeTask", ProxyTarget::ExtHostTaskService.GetTargetPrefix());
						match IPCProvider
							.SendRequestToSideCar("cocoon-main".to_string(), Method, json!([task]), 30000)
							.await
						{
							Ok(value) => Ok(value),
							Err(error) => {
								dev_log!(
									"ipc",
									"warn: [Task.Execute] extension did not answer ({:?}); returning null",
									error
								);
								Ok(json!(null))
							},
						}
					})
				};
			Ok(Box::new(effect))
		},

		// Authentication — GetSession/GetAccounts
		//
		// Auth sessions live in the extension that registered as the provider
		// (`vscode.authentication.registerAuthenticationProvider`), hosted in
		// Cocoon. Mountain forwards the request through ExtHostAuthentication.
		// Failing sidecars resolve to `null` / `[]` so Copilot / GitHub can
		// proceed in an unauthenticated path rather than rejecting at startup.
		"Authentication.GetSession" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider_id = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let scopes = Parameters.get(1).cloned().unwrap_or(json!([]));
						let options = Parameters.get(2).cloned().unwrap_or(json!({}));
						let IPCProvider:Arc<dyn IPCProviderTrait> = run_time.Environment.Require();
						let Method = format!("{}$getSession", ProxyTarget::ExtHostAuthentication.GetTargetPrefix());
						match IPCProvider
							.SendRequestToSideCar(
								"cocoon-main".to_string(),
								Method,
								json!([provider_id, scopes, options]),
								5000,
							)
							.await
						{
							Ok(value) => Ok(value),
							Err(error) => {
								dev_log!(
									"ipc",
									"warn: [Authentication.GetSession] extension did not answer ({:?}); returning null",
									error
								);
								Ok(json!(null))
							},
						}
					})
				};
			Ok(Box::new(effect))
		},

		"Authentication.GetAccounts" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider_id = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let IPCProvider:Arc<dyn IPCProviderTrait> = run_time.Environment.Require();
						let Method = format!("{}$getAccounts", ProxyTarget::ExtHostAuthentication.GetTargetPrefix());
						match IPCProvider
							.SendRequestToSideCar("cocoon-main".to_string(), Method, json!([provider_id]), 5000)
							.await
						{
							Ok(value) => Ok(value),
							Err(error) => {
								dev_log!(
									"ipc",
									"warn: [Authentication.GetAccounts] extension did not answer ({:?}); returning []",
									error
								);
								Ok(json!([]))
							},
						}
					})
				};
			Ok(Box::new(effect))
		},

		// Clipboard — Read/Write, backed by the cross-platform `arboard` crate
		// so we don't depend on the optional tauri-plugin-clipboard-manager.
		// arboard's API is blocking; we dispatch to tokio's blocking pool to
		// keep the async scheduler responsive.
		"Clipboard.Read" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let result = tokio::task::spawn_blocking(|| -> Result<String, String> {
							let mut Clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
							Clipboard.get_text().map_err(|e| e.to_string())
						})
						.await
						.map_err(|e| format!("Clipboard.Read join error: {}", e))?;
						match result {
							Ok(text) => Ok(json!(text)),
							Err(e) => {
								// Empty clipboard is reported as an error by arboard on
								// some platforms; treat those as empty-string instead of
								// bubbling to the extension.
								if e.contains("empty") || e.contains("Empty") {
									Ok(json!(""))
								} else {
									dev_log!("ipc", "warn: [Clipboard.Read] {}", e);
									Err(e)
								}
							},
						}
					})
				};
			Ok(Box::new(effect))
		},

		"Clipboard.Write" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let text = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let text_len = text.len();
						let result = tokio::task::spawn_blocking(move || -> Result<(), String> {
							let mut Clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
							Clipboard.set_text(text).map_err(|e| e.to_string())
						})
						.await
						.map_err(|e| format!("Clipboard.Write join error: {}", e))?;
						result.map(|()| {
							dev_log!("ipc", "[Clipboard.Write] wrote {} byte(s)", text_len);
							json!(null)
						})
					})
				};
			Ok(Box::new(effect))
		},

		// NativeHost — OpenExternal. Uses the `open` crate to hand the URI to
		// the OS default handler (xdg-open / `open` on macOS / ShellExecute on
		// Windows). We reject unsafe schemes (javascript:, data:, vbscript:,
		// file: written as an arbitrary path) because they can execute code
		// with host privileges, per Ladder §6.3 risk checklist.
		"NativeHost.OpenExternal" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let uri = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let lower = uri.to_ascii_lowercase();
						const BlockedSchemes:&[&str] = &["javascript:", "data:", "vbscript:", "file:"];
						for scheme in BlockedSchemes {
							if lower.starts_with(scheme) {
								dev_log!("ipc", "warn: [NativeHost.OpenExternal] rejected scheme '{}': {}", scheme, uri);
								return Err(format!("NativeHost.OpenExternal: scheme '{}' is not allowed", scheme));
							}
						}
						if uri.is_empty() {
							return Err("NativeHost.OpenExternal: empty URI".to_string());
						}
						let uri_owned = uri.clone();
						let result = tokio::task::spawn_blocking(move || open::that_detached(uri_owned))
							.await
							.map_err(|e| format!("NativeHost.OpenExternal join error: {}", e))?;
						match result {
							Ok(()) => {
								dev_log!("ipc", "[NativeHost.OpenExternal] opened {}", uri);
								Ok(json!(true))
							},
							Err(e) => {
								dev_log!("ipc", "warn: [NativeHost.OpenExternal] failed uri={} error={}", uri, e);
								Err(e.to_string())
							},
						}
					})
				};
			Ok(Box::new(effect))
		},

		// Languages — GetAll
		//
		// Walk every scanned extension's `contributes.languages[]`, merge by
		// language id, and return the union. Returned shape matches the VS Code
		// contract: `[{ id, aliases, extensions, filenames, mimetypes,
		// configuration }]`. Unknown fields survive as `null` / `[]`.
		"Languages.GetAll" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						use std::collections::HashMap;

						let scanned = run_time
							.Environment
							.ApplicationState
							.Extension
							.ScannedExtensions
							.ScannedExtensions
							.clone();
						let Guard = match scanned.lock() {
							Ok(g) => g,
							Err(error) => {
								return Err(format!("Languages.GetAll: scanned-extensions lock poisoned: {}", error));
							},
						};

						let mut merged:HashMap<String, serde_json::Map<String, Value>> = HashMap::new();
						for Dto in Guard.values() {
							let Contributes = match Dto.Contributes.as_ref() {
								Some(c) => c,
								None => continue,
							};
							let Languages = Contributes.get("languages").and_then(Value::as_array);
							let Some(Languages) = Languages else { continue };
							for Entry in Languages {
								let Id = match Entry.get("id").and_then(Value::as_str) {
									Some(id) if !id.is_empty() => id.to_string(),
									_ => continue,
								};
								let Existing = merged.entry(Id.clone()).or_insert_with(|| {
									let mut seed = serde_json::Map::new();
									seed.insert("id".to_string(), json!(Id));
									seed.insert("aliases".to_string(), json!([]));
									seed.insert("extensions".to_string(), json!([]));
									seed.insert("filenames".to_string(), json!([]));
									seed.insert("filenamePatterns".to_string(), json!([]));
									seed.insert("mimetypes".to_string(), json!([]));
									seed.insert("configuration".to_string(), Value::Null);
									seed
								});
								let merge_array =
									|target:&mut serde_json::Map<String, Value>, key:&str, incoming:&Value| {
										let Some(incoming_arr) = incoming.get(key).and_then(Value::as_array) else {
											return;
										};
										let bucket = target.entry(key.to_string()).or_insert_with(|| json!([]));
										if let Some(bucket_arr) = bucket.as_array_mut() {
											for v in incoming_arr {
												if !bucket_arr.iter().any(|e| e == v) {
													bucket_arr.push(v.clone());
												}
											}
										}
									};
								merge_array(Existing, "aliases", Entry);
								merge_array(Existing, "extensions", Entry);
								merge_array(Existing, "filenames", Entry);
								merge_array(Existing, "filenamePatterns", Entry);
								merge_array(Existing, "mimetypes", Entry);
								if Existing.get("configuration").map(Value::is_null).unwrap_or(true) {
									if let Some(cfg) = Entry.get("configuration") {
										Existing.insert("configuration".to_string(), cfg.clone());
									}
								}
							}
						}
						drop(Guard);

						let result:Vec<Value> = merged.into_values().map(Value::Object).collect();
						dev_log!("ipc", "[Languages.GetAll] returning {} languages", result.len());
						Ok(json!(result))
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
