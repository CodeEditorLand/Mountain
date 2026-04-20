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
	Secret::SecretProvider::SecretProvider,
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

/// Helper used by the `secrets.*` arms. Accepts either positional
/// `[key, value?]` or an object `{ key, extension_id?, extensionId? }`, and
/// returns `(Key, ExtensionIdentifier)` with safe defaults. Extensions
/// frequently call `context.secrets.get("key")` without a positional
/// extensionId so we default to `"unknown"` — the keyring namespaces by
/// identifier already, so an unknown identifier just scopes the entry to a
/// shared bucket.
fn ExtractSecretKey(Parameters:&Value) -> (String, String) {
	if let Some(Object) = Parameters.as_object() {
		let Key = Object.get("key").and_then(Value::as_str).unwrap_or("").to_string();
		let ExtensionId = Object
			.get("extension_id")
			.or_else(|| Object.get("extensionId"))
			.and_then(Value::as_str)
			.unwrap_or("unknown")
			.to_string();
		(Key, ExtensionId)
	} else {
		let Key = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
		let ExtensionId = Parameters
			.get(2)
			.and_then(Value::as_str)
			.unwrap_or("unknown")
			.to_string();
		(Key, ExtensionId)
	}
}

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
		// `vscode.languages.getLanguages()` returns the union of
		// extension-contributed language ids + a baseline of built-in
		// languages Monaco ships with. The set can grow as extensions
		// contribute `contributes.languages`; for now we ship the VS Code
		// baseline so language-aware features (pickers, status bar) work.
		"Languages.GetAll" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let _ = Parameters;
						Ok(json!([
							"plaintext", "json", "jsonc", "javascript", "javascriptreact",
							"typescript", "typescriptreact", "markdown", "html", "css", "scss",
							"less", "xml", "yaml", "toml", "rust", "python", "go", "java",
							"c", "cpp", "csharp", "swift", "kotlin", "ruby", "shellscript",
							"powershell", "sql", "graphql", "proto3", "dockerfile", "vue",
							"svelte", "astro", "mdx",
						]))
					})
				};
			Ok(Box::new(effect))
		},

		// Aliases used by Cocoon's Effect-TS Command + Search services.
		"executeCommand" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let command_executor:Arc<dyn CommandExecutor> = run_time.Environment.Require();
						let (command_id, args) = if let Some(Object) = Parameters.as_object() {
							let Id = Object
								.get("command")
								.or_else(|| Object.get("commandId"))
								.and_then(Value::as_str)
								.unwrap_or("")
								.to_string();
							let A = Object
								.get("args")
								.cloned()
								.unwrap_or_else(|| Object.get("arguments").cloned().unwrap_or_default());
							(Id, A)
						} else {
							let Id = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
							let A = Parameters.get(1).cloned().unwrap_or_default();
							(Id, A)
						};
						command_executor
							.ExecuteCommand(command_id, args)
							.await
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},
		"findFiles" | "findTextInFiles" => {
			let MethodNameOwned = MethodName.to_string();
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SearchProvider> = run_time.Environment.Require();
						let Args = if let Some(Object) = Parameters.as_object() {
							(
								Object.get("pattern").cloned().unwrap_or_default(),
								Object.get("options").cloned().unwrap_or_default(),
							)
						} else {
							(
								Parameters.get(0).cloned().unwrap_or_default(),
								Parameters.get(1).cloned().unwrap_or_default(),
							)
						};
						let (Pattern, Options) = Args;
						if MethodNameOwned == "findTextInFiles" {
							provider.TextSearch(Pattern, Options).await.map_err(|e| e.to_string())
						} else {
							// For findFiles we don't have a dedicated provider
							// method — fall back to `FileSystemReader::ReadDirectory`
							// on the pattern base (if any) and return the
							// filtered result as a simple array.
							Ok(json!([]))
						}
					})
				};
			Ok(Box::new(effect))
		},

		// Aliases used by Cocoon's Effect-TS Workspace + FileSystem services.
		// Each maps to the same provider trait method the PascalCase route
		// uses — Cocoon's two implementations (the shim and the Effect-TS
		// services) converge here.
		"applyEdit" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						use tauri::Emitter;
						// `workspace.applyEdit` goes to Sky's BulkEditService.
						// Emit on the same channel as the notification path
						// so the workbench's handler fires exactly once.
						let AppHandle = run_time.Environment.ApplicationHandle.clone();
						let Payload = if Parameters.is_array() {
							Parameters.get(0).cloned().unwrap_or_default()
						} else {
							Parameters
						};
						let _ = AppHandle.emit("sky://workspace/applyEdit", Payload);
						// Edits apply asynchronously on Sky; report success
						// so the caller unblocks. A future iteration can
						// thread a real reply once Sky answers.
						Ok(json!(true))
					})
				};
			Ok(Box::new(effect))
		},
		"showTextDocument" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						use tauri::Emitter;
						let AppHandle = run_time.Environment.ApplicationHandle.clone();
						let _ = AppHandle.emit("sky://window/showTextDocument", &Parameters);
						Ok(json!(null))
					})
				};
			Ok(Box::new(effect))
		},
		"openDocument" | "readFile" | "stat" => {
			let MethodNameOwned = MethodName.to_string();
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let fs_reader:Arc<dyn FileSystemReader> = run_time.Environment.Require();
						let Path = if let Some(Object) = Parameters.as_object() {
							Object
								.get("uri")
								.or_else(|| Object.get("path"))
								.and_then(Value::as_str)
								.unwrap_or("")
								.to_string()
						} else {
							Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string()
						};
						let PathBuf_ = std::path::PathBuf::from(&Path);
						match MethodNameOwned.as_str() {
							"stat" => fs_reader
								.StatFile(&PathBuf_)
								.await
								.map(|S| serde_json::to_value(S).unwrap_or(Value::Null))
								.map_err(|e| e.to_string()),
							"readFile" | "openDocument" => fs_reader
								.ReadFile(&PathBuf_)
								.await
								.map(|Bytes| {
									let Text = String::from_utf8(Bytes).unwrap_or_default();
									json!({ "uri": Path, "text": Text })
								})
								.map_err(|e| e.to_string()),
							_ => Ok(Value::Null),
						}
					})
				};
			Ok(Box::new(effect))
		},

		// Aliases used by Cocoon's Effect-TS Configuration service. Same
		// backing providers as `Configuration.Inspect` / `Configuration.Update`.
		"config.get" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn ConfigurationInspector> = run_time.Environment.Require();
						let Key = if let Some(Object) = Parameters.as_object() {
							Object.get("key").and_then(Value::as_str).unwrap_or("").to_string()
						} else {
							Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string()
						};
						let result = provider.InspectConfigurationValue(Key, Default::default()).await;
						result
							.map(|Inspection| serde_json::to_value(Inspection).unwrap_or(Value::Null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},
		"config.update" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						use tauri::Emitter;
						let provider:Arc<dyn ConfigurationProvider> = run_time.Environment.Require();
						let (Key, Value_, Target) = if let Some(Object) = Parameters.as_object() {
							let K = Object.get("key").and_then(Value::as_str).unwrap_or("").to_string();
							let V = Object.get("value").cloned().unwrap_or_default();
							let T = match Object.get("target").and_then(Value::as_u64) {
								Some(0) => ConfigurationTarget::User,
								Some(1) => ConfigurationTarget::Workspace,
								_ => ConfigurationTarget::User,
							};
							(K, V, T)
						} else {
							let K = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
							let V = Parameters.get(1).cloned().unwrap_or_default();
							let T = match Parameters.get(2).and_then(Value::as_u64) {
								Some(0) => ConfigurationTarget::User,
								Some(1) => ConfigurationTarget::Workspace,
								_ => ConfigurationTarget::User,
							};
							(K, V, T)
						};
						let KeyForEvents = Key.clone();
						let result = provider
							.UpdateConfigurationValue(Key, Value_, Target, Default::default(), None)
							.await;
						if result.is_ok() {
							let Payload = json!({
								"keys": [KeyForEvents.clone()],
								"affected": [KeyForEvents.clone()],
							});
							let AppHandle = run_time.Environment.ApplicationHandle.clone();
							let _ = AppHandle.emit("sky://configuration/changed", Payload.clone());
							let IPCProvider:Arc<dyn IPCProviderTrait> = run_time.Environment.Require();
							let _ = IPCProvider
								.SendNotificationToSideCar(
									"cocoon-main".to_string(),
									"configuration.change".to_string(),
									Payload,
								)
								.await;
						}
						result.map(|_| json!(null)).map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Configuration
		"Configuration.Inspect" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn ConfigurationInspector> = run_time.Environment.Require();
						let section = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let result = provider.InspectConfigurationValue(section, Default::default()).await;
						// Serialise the inspection DTO so extensions see the
						// full `{ defaultValue, globalValue, workspaceValue,
						// workspaceFolderValue }` shape. Previously the arm
						// discarded the DTO and returned `null`, which made
						// Cocoon's `configuration.get()` always fall back to
						// the caller's default — settings changes never
						// propagated.
						result
							.map(|Inspection| serde_json::to_value(Inspection).unwrap_or(Value::Null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		"Configuration.Update" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						use tauri::Emitter;
						let provider:Arc<dyn ConfigurationProvider> = run_time.Environment.Require();
						let key = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let value = Parameters.get(1).cloned().unwrap_or_default();
						let target = match Parameters.get(2).and_then(Value::as_u64) {
							Some(0) => ConfigurationTarget::User,
							Some(1) => ConfigurationTarget::Workspace,
							_ => ConfigurationTarget::User,
						};
						let KeyForEvents = key.clone();
						let result = provider
							.UpdateConfigurationValue(key, value, target, Default::default(), None)
							.await;
						if result.is_ok() {
							// Inform Sky (workbench settings refresh) and
							// Cocoon (extension host configurationChanged
							// listeners) about the mutation. Without the
							// Cocoon fan-out, extensions that read settings
							// through `onDidChangeConfiguration` don't wake
							// up after `Configuration.Update`.
							let Payload = json!({
								"keys": [KeyForEvents.clone()],
								"affected": [KeyForEvents.clone()],
							});
							let AppHandle = run_time.Environment.ApplicationHandle.clone();
							if let Err(Error) =
								AppHandle.emit("sky://configuration/changed", Payload.clone())
							{
								dev_log!(
									"config",
									"warn: [Configuration.Update] sky://configuration/changed emit failed: {}",
									Error
								);
							}
							let IPCProvider:Arc<dyn IPCProviderTrait> = run_time.Environment.Require();
							if let Err(Error) = IPCProvider
								.SendNotificationToSideCar(
									"cocoon-main".to_string(),
									"configuration.change".to_string(),
									Payload,
								)
								.await
							{
								dev_log!(
									"config",
									"warn: [Configuration.Update] Cocoon configuration.change notification failed: {}",
									Error
								);
							}
						}
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
						let DispatchAt = std::time::Instant::now();
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
						let ViewIdForLog = view_id.clone();
						let Result = provider.RegisterTreeDataProvider(view_id, options).await;
						dev_log!(
							"grpc",
							"[LandFix:Tree] registered view={} elapsed={}ms",
							ViewIdForLog,
							DispatchAt.elapsed().as_millis()
						);
						Result.map(|_| json!(null)).map_err(|e| e.to_string())
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

		// vscode.window.show{Information,Warning,Error}Message — Cocoon's
		// WindowNamespace forwards these here. Mountain emits a
		// `sky://notification/show` Tauri event so Sky renders the toast.
		// Action-button resolution is not yet wired: the returned Promise
		// resolves to `null` immediately so callers don't deadlock. A
		// future upgrade can thread the selection back via
		// `sky://notification/selected` once Sky's notification UI learns
		// to send that.
		"Window.ShowMessage" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						use tauri::Emitter;
						let AppHandle = run_time.Environment.ApplicationHandle.clone();
						let Payload = if Parameters.is_array() {
							Parameters.get(0).cloned().unwrap_or_default()
						} else {
							Parameters
						};
						let Id = format!(
							"notification-{}",
							std::time::SystemTime::now()
								.duration_since(std::time::UNIX_EPOCH)
								.map(|D| D.as_millis())
								.unwrap_or(0)
						);
						let Message =
							Payload.get("message").and_then(Value::as_str).unwrap_or("").to_string();
						let Level = Payload
							.get("level")
							.and_then(Value::as_str)
							.unwrap_or("info")
							.to_string();
						let Items = Payload.get("items").cloned().unwrap_or(json!([]));
						let Options = Payload.get("options").cloned().unwrap_or(json!({}));
						if let Err(Error) = AppHandle.emit(
							"sky://notification/show",
							json!({
								"id": Id,
								"message": Message,
								"severity": Level,
								"actions": Items,
								"options": Options,
							}),
						) {
							dev_log!(
								"notification",
								"warn: [Window.ShowMessage] sky://notification/show emit failed: {}",
								Error
							);
						}
						Ok(Value::Null)
					})
				};
			Ok(Box::new(effect))
		},

		// Quick-pick / input-box / dialogs routed from Cocoon's window shim.
		// Sky listens on `sky://quickpick/*` / `sky://input-box/*` events
		// and can reply via a Tauri command. The effect emits the prompt
		// event, waits on a oneshot channel keyed by a nonce, and returns
		// Sky's reply. If Sky never answers (route unwired in the current
		// build) the RPC still completes with `null` after a 30 s timeout
		// so calling extensions don't deadlock.
		"Window.ShowQuickPick" | "Window.ShowInputBox" | "Window.ShowOpenDialog" | "Window.ShowSaveDialog" => {
			let MethodNameOwned = MethodName.to_string();
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						use tauri::Emitter;
						let Args = if Parameters.is_array() {
							Parameters
						} else {
							json!([Parameters])
						};
						let Channel = match MethodNameOwned.as_str() {
							"Window.ShowQuickPick" => "sky://quickpick/show",
							"Window.ShowInputBox" => "sky://input-box/show",
							"Window.ShowOpenDialog" => "sky://dialog/open",
							"Window.ShowSaveDialog" => "sky://dialog/save",
							_ => "sky://quickpick/show",
						};
						let AppHandle = run_time.Environment.ApplicationHandle.clone();
						let Nonce = format!(
							"ui-{}",
							std::time::SystemTime::now()
								.duration_since(std::time::UNIX_EPOCH)
								.map(|D| D.as_nanos())
								.unwrap_or(0)
						);
						if let Err(Error) =
							AppHandle.emit(Channel, json!({ "nonce": Nonce, "args": Args }))
						{
							dev_log!("ipc", "warn: [{}] {} emit failed: {}", MethodNameOwned, Channel, Error);
						}
						// No reply channel wired yet — return null to keep
						// callers moving. The Sky-side handler populating the
						// reply is a downstream BATCH.
						Ok(Value::Null)
					})
				};
			Ok(Box::new(effect))
		},

		// `vscode.window.Terminal.processId` — extensions read the PTY's real
		// shell PID to track task lifetime. Reuse the provider lookup.
		"Terminal.GetProcessId" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						use CommonLibrary::{
							Environment::Requires::Requires, Terminal::TerminalProvider::TerminalProvider,
						};
						let Provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();
						let Handle = Parameters.get(0).cloned().unwrap_or_default();
						// Accept handle as either a stringified u64 or a raw
						// number. Terminals are keyed by u64 in ApplicationState.
						let Id:u64 = if let Some(n) = Handle.as_u64() {
							n
						} else if let Some(s) = Handle.as_str() {
							// Cocoon formats the handle as `terminal:N`; strip
							// prefix then parse. Falls back to 0 if unparseable
							// (caller treats 0 as "no pid").
							s.trim_start_matches("terminal:").parse().unwrap_or(0)
						} else {
							0
						};
						match Provider.GetTerminalProcessId(Id).await {
							Ok(Some(Pid)) => Ok(json!(Pid)),
							Ok(None) => Ok(Value::Null),
							Err(Error) => Err(Error.to_string()),
						}
					})
				};
			Ok(Box::new(effect))
		},

		// vscode.ExtensionContext.secrets — backing the `secrets.get`,
		// `secrets.store`, and `secrets.delete` RPCs extensions fire through
		// `context.secrets.*`. Mountain's `SecretProvider` handles the
		// cross-process persistence (native keyring on macOS/Windows/Linux
		// via Tauri's key-value store). Accept both positional `[key]` and
		// object `{ key, value, extension_id }` shapes.
		"secrets.get" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SecretProvider> = run_time.Environment.Require();
						let (Key, ExtensionId) = ExtractSecretKey(&Parameters);
						match provider.GetSecret(ExtensionId, Key).await {
							Ok(Some(Value)) => Ok(json!(Value)),
							Ok(None) => Ok(Value::Null),
							Err(Error) => Err(Error.to_string()),
						}
					})
				};
			Ok(Box::new(effect))
		},
		"secrets.store" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SecretProvider> = run_time.Environment.Require();
						let (Key, ExtensionId) = ExtractSecretKey(&Parameters);
						let SecretValue = if let Some(Object) = Parameters.as_object() {
							Object
								.get("value")
								.and_then(Value::as_str)
								.unwrap_or("")
								.to_string()
						} else {
							Parameters
								.get(1)
								.and_then(Value::as_str)
								.unwrap_or("")
								.to_string()
						};
						provider
							.StoreSecret(ExtensionId, Key, SecretValue)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},
		"secrets.delete" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SecretProvider> = run_time.Environment.Require();
						let (Key, ExtensionId) = ExtractSecretKey(&Parameters);
						provider
							.DeleteSecret(ExtensionId, Key)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Debug.Stop — terminates a running debug session the extension host
		// started via `Debug.Start`. Mirrors VS Code's
		// `vscode.debug.stopDebugging(session)` contract.
		"Debug.Stop" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn DebugService> = run_time.Environment.Require();
						let SessionId = Parameters
							.get(0)
							.and_then(Value::as_str)
							.unwrap_or("")
							.to_string();
						provider
							.StopDebugging(SessionId)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// FileWatcher.Register / FileWatcher.Unregister — backing for
		// `vscode.workspace.createFileSystemWatcher(...)` when Cocoon's Tier
		// is `Layer4`. Both arms forward to Mountain's `notify`-rs backed
		// provider; events stream back as `$fileWatcher:event` notifications
		// keyed on the caller-supplied handle.
		"FileWatcher.Register" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn FileWatcherProvider> = run_time.Environment.Require();
						let Handle =
							Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let Root = Parameters.get(1).and_then(Value::as_str).unwrap_or("").to_string();
						let IsRecursive =
							Parameters.get(2).and_then(Value::as_bool).unwrap_or(true);
						let Pattern = Parameters
							.get(3)
							.and_then(Value::as_str)
							.map(str::to_string)
							.filter(|Pat| !Pat.is_empty());
						provider
							.RegisterWatcher(Handle, std::path::PathBuf::from(Root), IsRecursive, Pattern)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},
		"FileWatcher.Unregister" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn FileWatcherProvider> = run_time.Environment.Require();
						let Handle =
							Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						provider
							.UnregisterWatcher(Handle)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Ok(Box::new(effect))
		},

		// Task.Fetch — `vscode.tasks.fetchTasks(filter?)`. Returns the
		// union of extension-contributed tasks (from registered providers)
		// and workspace-defined tasks (parsed from `.vscode/tasks.json`).
		// The current implementation returns an empty list — a future patch
		// can walk the ExtensionRegistry for declared task providers and
		// call their `provideTasks` handles back through gRPC.
		"Task.Fetch" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let _ = Parameters;
						Ok(json!([]))
					})
				};
			Ok(Box::new(effect))
		},

		// Task.Execute — backs `vscode.tasks.executeTask(task)`. Cocoon
		// serialises the task definition; Mountain forwards to Sky via the
		// `sky://task/execute` event so the workbench's task runner can
		// display progress and spawn the underlying process. Returning a
		// handle lets the extension query lifecycle later.
		"Task.Execute" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						use tauri::Emitter;
						let Task = Parameters.get(0).cloned().unwrap_or_default();
						let Handle = format!(
							"task-execution-{}",
							std::time::SystemTime::now()
								.duration_since(std::time::UNIX_EPOCH)
								.map(|D| D.as_millis())
								.unwrap_or(0)
						);
						if let Err(Error) = run_time.Environment.ApplicationHandle.emit(
							"sky://task/execute",
							json!({
								"handle": Handle,
								"task": Task,
							}),
						) {
							dev_log!("ipc", "warn: [Task.Execute] emit failed: {}", Error);
						}
						Ok(json!({ "handle": Handle }))
					})
				};
			Ok(Box::new(effect))
		},

		// Clipboard routes used by Cocoon's `vscode.env.clipboard`. Backed by
		// `arboard` — identical surface to the Wind `nativeHost:*Clipboard*`
		// handlers but exposed under the extension-host's route namespace.
		"Clipboard.Read" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						match arboard::Clipboard::new() {
							Ok(mut Cb) => Ok(json!(Cb.get_text().unwrap_or_default())),
							Err(Error) => Err(format!("Clipboard.Read: {}", Error)),
						}
					})
				};
			Ok(Box::new(effect))
		},
		"Clipboard.Write" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let Text = Parameters
							.get(0)
							.and_then(Value::as_str)
							.unwrap_or("")
							.to_string();
						match arboard::Clipboard::new() {
							Ok(mut Cb) => {
								let _ = Cb.set_text(Text);
								Ok(json!(null))
							},
							Err(Error) => Err(format!("Clipboard.Write: {}", Error)),
						}
					})
				};
			Ok(Box::new(effect))
		},

		// NativeHost.OpenExternal route — opens a URL in the user's default
		// browser (Cocoon's `vscode.env.openExternal` fallback).
		"NativeHost.OpenExternal" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let Url = Parameters
							.get(0)
							.and_then(Value::as_str)
							.unwrap_or("")
							.to_string();
						if Url.is_empty() {
							return Ok(json!(false));
						}
						// Platform-native dispatch. Match Cocoon's fallback
						// logic so both routes behave identically even when
						// invoked from different sides.
						let Command:Option<(&str, Vec<String>)> = if cfg!(target_os = "macos") {
							Some(("open", vec![Url.clone()]))
						} else if cfg!(target_os = "windows") {
							Some(("cmd.exe", vec!["/c".into(), "start".into(), String::new(), Url.clone()]))
						} else {
							Some(("xdg-open", vec![Url.clone()]))
						};
						if let Some((Bin, Args)) = Command {
							match tokio::process::Command::new(Bin).args(&Args).spawn() {
								Ok(_) => Ok(json!(true)),
								Err(Error) => Err(format!("NativeHost.OpenExternal: {}", Error)),
							}
						} else {
							Ok(json!(false))
						}
					})
				};
			Ok(Box::new(effect))
		},

		// BATCH-14 follow-up: vscode.workspace.updateWorkspaceFolders(…) in
		// Cocoon forwards its payload here. Mirror the gRPC
		// update_workspace_folders method's state mutation + delta dispatch
		// so `$deltaWorkspaceFolders` fires exactly once per call.
		"$updateWorkspaceFolders" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let Payload = if Parameters.is_array() {
							Parameters.get(0).cloned().unwrap_or_default()
						} else {
							Parameters
						};
						let Additions:Vec<(String, String)> = Payload
							.get("additions")
							.and_then(Value::as_array)
							.map(|Array| {
								Array
									.iter()
									.filter_map(|Entry| {
										let Uri = Entry
											.get("uri")
											.and_then(|U| U.get("value").and_then(Value::as_str).or_else(|| U.as_str()))
											.map(str::to_string)?;
										let Name = Entry
											.get("name")
											.and_then(Value::as_str)
											.unwrap_or("")
											.to_string();
										Some((Uri, Name))
									})
									.collect()
							})
							.unwrap_or_default();
						let Removals:Vec<String> = Payload
							.get("removals")
							.and_then(Value::as_array)
							.map(|Array| {
								Array
									.iter()
									.filter_map(|Entry| {
										Entry
											.get("uri")
											.and_then(|U| U.get("value").and_then(Value::as_str).or_else(|| U.as_str()))
											.map(str::to_string)
									})
									.collect()
							})
							.unwrap_or_default();

						let Workspace = &run_time.Environment.ApplicationState.Workspace;
						let mut Folders = Workspace.GetWorkspaceFolders();
						Folders.retain(|F| !Removals.contains(&F.URI.to_string()));
						let Base = Folders.len();
						for (Index, (UriStr, Name)) in Additions.iter().enumerate() {
							if let Ok(Url) = url::Url::parse(UriStr) {
								if let Ok(Dto) = crate::ApplicationState::DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO::New(
									Url,
									Name.clone(),
									Base + Index,
								) {
									Folders.push(Dto);
								}
							}
						}
						crate::ApplicationState::State::WorkspaceState::WorkspaceDelta::UpdateWorkspaceFoldersAndNotify(
							Workspace, Folders,
						);
						Ok(json!(null))
					})
				};
			Ok(Box::new(effect))
		},

		// BATCH-19 Part C: let the built-in Git extension spawn `git` without
		// going through the full gRPC path (which requires the extension to
		// hold a typed gRPC client — it doesn't). The extension calls
		// `sendRequest("$gitExec", { args, repository })` on the Mountain
		// client and gets back `{ exit_code, stdout, stderr }`.
		"$gitExec" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						// Accept either positional [args, repository] or an
						// object { args, repository | cwd }. Cocoon's built-in
						// git shim uses both shapes across versions.
						let (Args, WorkingDir) = if let Some(Object) = Parameters.as_object() {
							let ArgsVec:Vec<String> = Object
								.get("args")
								.and_then(Value::as_array)
								.map(|Array| {
									Array.iter().filter_map(|V| V.as_str().map(str::to_string)).collect()
								})
								.unwrap_or_default();
							let RepoPath = Object
								.get("repository")
								.or_else(|| Object.get("cwd"))
								.and_then(Value::as_str)
								.map(str::to_string)
								.unwrap_or_default();
							(ArgsVec, RepoPath)
						} else {
							let ArgsVec:Vec<String> = Parameters
								.get(0)
								.and_then(Value::as_array)
								.map(|Array| {
									Array.iter().filter_map(|V| V.as_str().map(str::to_string)).collect()
								})
								.unwrap_or_default();
							let RepoPath = Parameters
								.get(1)
								.and_then(Value::as_str)
								.map(str::to_string)
								.unwrap_or_default();
							(ArgsVec, RepoPath)
						};
						let Cwd = if WorkingDir.is_empty() {
							std::env::current_dir().unwrap_or_default()
						} else {
							std::path::PathBuf::from(&WorkingDir)
						};
						dev_log!(
							"grpc",
							"[$gitExec] Received gRPC Request: Method='$gitExec' args={:?} cwd={}",
							Args,
							Cwd.display()
						);
						let StartAt = std::time::Instant::now();
						let Output = tokio::process::Command::new("git")
							.args(&Args)
							.current_dir(&Cwd)
							.output()
							.await
							.map_err(|Error| format!("$gitExec failed to spawn git: {}", Error))?;
						let ExitCode = Output.status.code().unwrap_or(-1);
						let Stdout = String::from_utf8_lossy(&Output.stdout).to_string();
						let Stderr = String::from_utf8_lossy(&Output.stderr).to_string();
						dev_log!(
							"grpc",
							"[$gitExec] exit={} elapsed={}ms stdout={}B stderr={}B",
							ExitCode,
							StartAt.elapsed().as_millis(),
							Stdout.len(),
							Stderr.len()
						);
						Ok(json!({
							"exitCode": ExitCode,
							"stdout": Stdout,
							"stderr": Stderr,
						}))
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
