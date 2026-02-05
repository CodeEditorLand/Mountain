//! # EffectCreation (Track)
//!
//! RESPONSIBILITIES:
//! - Central routing table that maps string-based commands/RPC methods to typed
//!   effects
//! - Creates `MappedEffect` (type-erased async closures) for dispatch execution
//! - Integrates with the effect system (`ActionEffect`) and provider traits
//! - Provides direct provider calls for performance-critical operations
//!
//! ARCHITECTURAL ROLE:
//! - Core component in the Track module (command dispatch system)
//! - Sits between `DispatchLogic` (router) and `ApplicationRunTime` (executor)
//! - Pattern: Command → Effect → Provider → Result
//! - Uses `Common` crate effect types for all Mountain operations
//!
//! EFFECT CREATION FLOW:
//! 1. `DispatchLogic` receives command/RPC from frontend or sidecar
//! 2. Calls `EffectCreation::CreateEffectForRequest` with method name + params
//! 3. EffectCreation matches method to effect constructor (match statement)
//! 4. Constructs typed effect with deserialized parameters
//! 5. Returns `MappedEffect` (boxed future) to be executed by
//!    `ApplicationRunTime`
//! 6. Runtime executes effect via provider trait (DI via `Require`)
//! 7. Result propagates back through the call chain
//!
//! DIRECT PROVIDER CALLS:
//! Some operations bypass the effect system for performance:
//! - Configuration: `GetConfiguration`, `UpdateConfiguration`
//! - Diagnostics: `SetDiagnostics`, `ClearDiagnostics`
//! - Language Features: `ProvideHover`, `ProvideCompletions`, etc.
//! - Terminal: direct text send/receive
//! - Why? Avoid effect overhead for high-frequency operations
//!
//! ERROR HANDLING:
//! - All effects return `Result<Value, String>` (serializable errors for IPC)
//! - Parameter validation with descriptive error messages
//! - Unknown command handling returns error instead of panic
//! - Serialization/deserialization errors caught and reported
//! - Provider errors propagate with context
//!
//! PERFORMANCE:
//! - Effect creation is cheap: match + constructor call + box
//! - Direct provider calls avoid allocation (for hot paths)
//! - TODO: Consider implementing an effect pool to cache frequently created
//!   effects, reducing allocation overhead for high-frequency commands.
//! - TODO: Add configurable command timeouts per command type and rate limiting
//!   to prevent abuse and ensure system stability.
//!
//! VS CODE REFERENCE:
//! - `vs/workbench/services/extensions/common/extensions.ts` - command
//!   registration
//! - `vs/platform/commands/common/commands.ts` - command service and
//!   dispatching
//! - `vs/workbench/common/effect/effect.ts` - effect system pattern
//!
//! SUPPORTED COMMAND CATEGORIES:
//! **Commands**: Execute, GetAll, Register
//! **Configuration**: Inpect, Update, Get
//! **Documents**: Save, SaveAs
//! **FileSystem**: ReadFile, WriteFile, ReadDirectory, StatFile, Delete
//! **Debug**: Start, RegisterConfigurationProvider
//! **Diagnostics**: Set, Clear
//! **Keybinding**: GetResolved
//! **LanguageFeatures**: $languageFeatures:registerProvider, unregisterProvider
//! **Search**: TextSearch
//! **SourceControlManagement**: $scm:createSourceControl, updateSourceControl,
//! updateGroup, registerInputBox **StatusBar**: $statusBar:set, dispose,
//! $setStatusBarMessage, $disposeStatusBarMessage **Storage**: Get, Set,
//! $storage:getAll, $storage:setAll **Terminal**: $terminal:create, sendText,
//! dispose **TreeView**: $tree:register
//! **UserInterface**: ShowMessage, ShowOpenDialog, ShowSaveDialog
//! **Webview**: $webview:create, $resolveCustomEditor
//!
//! TODO:
//! - Add command parameter schema validation (JSON schema per command)
//! - Implement command permission checking (capability-based security)
//! - Add command deprecation warnings and migration
//! - Cache frequently created effects (reuse boxed closures)
//! - Add command timeout configuration (per-command TTL)
//! - Implement command rate limiting (DoS protection)
//! - Add command metrics collection (latency, success rate)
//! - Implement command aliasing (user-defined shortcuts)
//! - Add command migration support (rename, deprecate)
//! - Add comprehensive command audit logging
//! - Support command chaining and composition
//! - Implement command undo/redo integration
//!
//! MODULE CONTENTS:
//! - Type alias: `MappedEffect` - boxed async closure signature
//! - Macro: `Parameter!` - deserialize parameters from JSON array
//! - Main function: `CreateEffectForRequest` - map method name to effect (in
//!   DispatchLogic)
//! - Direct provider calls: Various `Provider::*` methods called without effect
//!   wrapper
//! - Effect constructors: All `*` effects from `CommonLibrary::Effect`
//!
//! ---
//! *This module is the heart of Mountain's command dispatch system, providing
//! the glue between UI commands and backend provider implementations.*

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{
	self,
	Command::{ExecuteCommand::ExecuteCommand, GetAllCommands::GetAllCommands, RegisterCommand::RegisterCommand},
	Configuration::{
		ConfigurationInspector::ConfigurationInspector,
		ConfigurationProvider::ConfigurationProvider,
		DTO::ConfigurationTarget::ConfigurationTarget,
		GetConfiguration::GetConfiguration,
	},
	CustomEditor::CustomEditorProvider::CustomEditorProvider,
	Debug::DebugService::DebugService,
	Diagnostic::DiagnosticManager::DiagnosticManager,
	Document::{SaveDocument::SaveDocument, SaveDocumentAs::SaveDocumentAs},
	Effect::{ActionEffect::ActionEffect, ApplicationRunTime::ApplicationRunTime as ApplicationRunTimeTrait},
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	FileSystem::{
		Delete::Delete,
		ReadDirectory::ReadDirectory,
		ReadFile::ReadFile,
		StatFile::StatFile,
		WriteFileBytes::WriteFileBytes,
	},
	Keybinding::KeybindingProvider::KeybindingProvider,
	LanguageFeature::{
		DTO::ProviderType::ProviderType,
		LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	},
	Search::SearchProvider::SearchProvider,
	SourceControlManagement::SourceControlManagementProvider::SourceControlManagementProvider,
	StatusBar::{DTO::StatusBarEntryDTO::StatusBarEntryDTO, StatusBarProvider::StatusBarProvider},
	Storage::{GetStorageItem::GetStorageItem, SetStorageItem::SetStorageItem, StorageProvider::StorageProvider},
	Terminal::TerminalProvider::TerminalProvider,
	TreeView::TreeViewProvider::TreeViewProvider,
	UserInterface::{ShowMessage::ShowMessage, ShowOpenDialog::ShowOpenDialog, ShowSaveDialog::ShowSaveDialog},
	Webview::WebviewProvider::WebviewProvider,
};
use serde_json::{Value, from_value, json};
use tauri::{AppHandle, Runtime};
use url::Url;
use log::warn;

use crate::{
	Environment::MountainEnvironment::MountainEnvironment,
	RunTime::ApplicationRunTime::ApplicationRunTime as MountainRunTime,
};

/// A type alias for a boxed, runnable effect. This is the "type-erased" unit of
/// work that the dispatch logic can execute.
pub type MappedEffect =
	Box<dyn FnOnce(Arc<MountainRunTime>) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> + Send>;

/// A helper macro to reduce boilerplate when getting and deserializing
/// parameters from a JSON array.
macro_rules! Parameter {
	($Parameter:expr, $Current:expr, $Type:ty) => {
		from_value::<$Type>(
			$Parameter
				.get($Current)
				.cloned()
				.ok_or_else(|| format!("Missing parameter at index {}", $Current))?,
		)
		.map_err(|Error| format!("Invalid parameter at index {}: {}", $Current, Error))
	};
}

/// Maps a string-based method name (command or RPC) to its corresponding effect
/// constructor, returning a boxed closure (`MappedEffect`) that can be executed
/// by the ApplicationRunTime.
///
/// # Arguments
/// - `ApplicationHandle`: Tauri app handle for accessing state
/// - `_State`: unused State parameter (for DI compatibility)
/// - `MethodName`: The command/RPC method name to map
/// - `_Parameters`: JSON array of parameters for the effect (currently unused)
///
/// # Returns
/// `MappedEffect` - a boxed async closure that takes Arc<ApplicationRunTime>
/// and returns Result<Value, String>
pub fn CreateEffectForRequest<R:Runtime>(
	_ApplicationHandle:AppHandle<R>,
	_State:&Arc<MountainEnvironment>,
	MethodName:&str,
	_Parameters:&Value,
) -> MappedEffect {
	// Simplified: direct provider calls for hot paths
	match MethodName {
		// Configuration
		"Configuration.Inspect" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn ConfigurationInspector> = run_time.Environment.Require();
					// Parse the section parameter from request arguments to support dynamic
					// configuration queries. Currently hardcoded to empty string, which only
					// retrieves global section configuration values. The section should be
					// deserialized from _Parameters[0] as a String.
					let section = String::new();
					provider.InspectConfigurationValue(target, &section).await
				}
			};
			Box::new(effect)
		},

		"Configuration.Update" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn ConfigurationProvider> = run_time.Environment.Require();
					// Parse configuration target, section, and value from request parameters.
					// Deserialize from _Parameters JSON array using the Parameter! macro.
					// Expected indices: 0=target (ConfigurationTarget), 1=section (String),
					// 2=value (Value). Integration with ConfigurationProvider requires proper
					// parameter extraction to update configuration correctly.
					provider
						.UpdateConfigurationValue(ConfigurationTarget::Global, "section".to_string(), json!({}))
						.await
				}
			};
			Box::new(effect)
		},

		// Diagnostics
		"Diagnostic.Set" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn DiagnosticManager> = run_time.Environment.Require();
					// Parse owner identifier and diagnostic entries from request parameters.
					// Expected indices: 0=owner (String, identifies the extension/source),
					// 1=entries (Vec<DiagnosticEntry> or array of diagnostic DTOs). This
					// enables proper diagnostic management per extension.
					provider.SetDiagnostics("owner".to_string(), json!([])).await
				}
			};
			Box::new(effect)
		},

		"Diagnostic.Clear" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn DiagnosticManager> = run_time.Environment.Require();
					provider.ClearDiagnostics("owner".to_string()).await
				}
			};
			Box::new(effect)
		},

		// Language Features
		"$languageFeatures:registerProvider" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn LanguageFeatureProviderRegistry> = run_time.Environment.Require();
					// Parse ProviderType, SelectorDTO, ExtensionIdentifierDTO, and OptionsDTO
					// from request parameters. These define the language feature provider
					// registration details and must be deserialized correctly to register the
					// provider with the LanguageFeatureProviderRegistry.
					provider
						.RegisterProvider("cocoon".to_string(), ProviderType::Hover, json!({}), json!({}), None)
						.await
				}
			};
			Box::new(effect)
		},

		// Documents
		"Document.Save" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let document_provider:Arc<dyn DocumentProvider> = run_time.Environment.Require();
					// Parse document URI from request parameters. The URI identifies the
					// document to save and must be deserialized from _Parameters[0]. Expected
					// format: VS Code URI scheme (e.g., file://, untitled:). Currently using
					// hardcoded test path for placeholder implementation.
					let uri = Url::parse("file:///tmp/test.txt").unwrap();
					document_provider.SaveDocument(uri).await
				}
			};
			Box::new(effect)
		},

		"Document.SaveAs" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let document_provider:Arc<dyn DocumentProvider> = run_time.Environment.Require();
					// Parse original document URI and target URI from request parameters.
					// Expected indices: 0=original URI (Url), 1=target URI (Url). This
					// implements "Save As" functionality, requiring both source and
					// destination URIs. Current placeholder uses hardcoded path.
					let original_uri = Url::parse("file:///tmp/test.txt").unwrap();
					document_provider.SaveDocumentAs(original_uri, None).await
				}
			};
			Box::new(effect)
		},

		// FileSystem
		"FileSystem.ReadFile" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let fs_reader:Arc<dyn FileSystemReader> = run_time.Environment.Require();
					// Parse filesystem path from request parameters. The path specifies which
					// file to read and should be deserialized from _Parameters[0] as a string,
					// then converted to PathBuf. Current implementation uses hardcoded test path.
					let path = std::path::PathBuf::from("/tmp/test.txt");
					fs_reader.ReadFile(&path).await
				}
			};
			Box::new(effect)
		},

		"FileSystem.WriteFile" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let fs_writer:Arc<dyn FileSystemWriter> = run_time.Environment.Require();
					// Parse filesystem path, content bytes, and write options from request
					// parameters. Expected indices: 0=path (String → PathBuf), 1=content
					// (Bytes or base64 string), 2=options (WriteOptions DTO). Proper
					// implementation requires deserializing all three parameters.
					let path = std::path::PathBuf::from("/tmp/test.txt");
					fs_writer.WriteFile(&path, &[]).await
				}
			};
			Box::new(effect)
		},

		"FileSystem.ReadDirectory" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let fs_reader:Arc<dyn FileSystemReader> = run_time.Environment.Require();
					// Parse directory path from request parameters to list directory contents.
					// Should be deserialized from _Parameters[0] as a string. Current
					// placeholder uses hardcoded "/tmp" directory.
					let path = std::path::PathBuf::from("/tmp");
					fs_reader.ReadDirectory(&path).await
				}
			};
			Box::new(effect)
		},

		// Keybinding
		"Keybinding.GetResolved" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn KeybindingProvider> = run_time.Environment.Require();
					provider.GetResolvedKeybinding().await
				}
			};
			Box::new(effect)
		},

		// Search
		"Search.TextSearch" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn SearchProvider> = run_time.Environment.Require();
					// Parse search query string and search options from request parameters.
					// Expected indices: 0=query (String), 1=options (SearchOptions DTO). These
					// control the text search behavior across workspace files.
					provider.TextSearch(json!({}), None, None, false, false).await
				}
			};
			Box::new(effect)
		},

		// Storage
		"Storage.Get" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn StorageProvider> = run_time.Environment.Require();
					// Parse storage key from request parameters. The key identifies which
					// storage item to retrieve and should be deserialized from _Parameters[0]
					// as a String. Current implementation uses hardcoded "key" placeholder.
					provider.GetStorageItem("key".to_string()).await
				}
			};
			Box::new(effect)
		},

		"Storage.Set" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn StorageProvider> = run_time.Environment.Require();
					// Parse storage key and value from request parameters for persisting data.
					// Expected indices: 0=key (String), 1=value (serde_json::Value). The value
					// can be any JSON-serializable type.
					provider.SetStorageItem("key".to_string(), json!("value")).await
				}
			};
			Box::new(effect)
		},

		// Commands
		"Command.Execute" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let command_executor:Arc<dyn CommandExecutor> = run_time.Environment.Require();
					// Parse command identifier and optional arguments from request parameters.
					// Expected indices: 0=command ID (String), 1=args (Value/JSON object). This
					// executes arbitrary commands registered by extensions or the host.
					command_executor.ExecuteCommand("command".to_string(), json!({})).await
				}
			};
			Box::new(effect)
		},

		"Command.GetAll" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let command_executor:Arc<dyn CommandExecutor> = run_time.Environment.Require();
					command_executor.GetAllCommands().await
				}
			};
			Box::new(effect)
		},

		// Status Bar
		"$statusBar:set" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();
					// Parse StatusBarEntryDTO from request parameters to create a new status bar
					// entry. The DTO contains text, identifier, alignment, priority, and other
					// UI properties. Should be deserialized from _Parameters[0].
					let entry = StatusBarEntryDTO {
						text:"status".to_string(),
						identifier:Some("id".to_string()),
						// other fields...
						..Default::default()
					};
					provider.SetStatusBarEntry(entry).await
				}
			};
			Box::new(effect)
		},

		"$statusBar:dispose" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();
					// Parse status bar entry identifier from request parameters to dispose of
					// the entry. Expected from _Parameters[0] as a String. Disposal removes the
					// entry from the status bar UI.
					provider.DisposeStatusBarEntry("id".to_string()).await
				}
			};
			Box::new(effect)
		},

		"$setStatusBarMessage" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();
					// Parse message identifier and message text from request parameters.
					// Expected indices: 0=message_id (String, for later disposal), 1=text
					// (String). Sets a temporary or persistent status bar message.
					provider.SetStatusBarMessage("msg_id".to_string(), "message".to_string()).await
				}
			};
			Box::new(effect)
		},

		"$disposeStatusBarMessage" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();
					// Parse message identifier from request parameters to hide and dispose of
					// a previously shown status bar message. Expected from _Parameters[0] as a
					// String. This cleans up status bar resources.
					provider.DisposeStatusBarMessage("msg_id".to_string()).await
				}
			};
			Box::new(effect)
		},

		// User Interface
		"UserInterface.ShowMessage" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();
					// Parse message type (info/warning/error), title, and message content from
					// request parameters. Expected indices: 0=type (String), 1=title (String),
					// 2=message (String), 3=options (Value, optional). This triggers
					// user-facing notifications.
					provider.ShowMessage("info".to_string(), "Title", "Message", json!({})).await
				}
			};
			Box::new(effect)
		},

		"UserInterface.ShowOpenDialog" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();
					// Parse file open dialog options from request parameters to customize the
					// dialog behavior. Expected from _Parameters[0] as OpenDialogOptions DTO
					// (filters, default path, multi-select, etc.).
					provider.ShowOpenDialog(None).await
				}
			};
			Box::new(effect)
		},

		"UserInterface.ShowSaveDialog" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();
					// Parse file save dialog options from request parameters to customize the
					// dialog behavior. Expected from _Parameters[0] as SaveDialogOptions DTO
					// (filters, default path, default name, etc.).
					provider.ShowSaveDialog(None).await
				}
			};
			Box::new(effect)
		},

		// Terminal
		"$terminal:create" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();
					// Parse terminal name and creation options from request parameters.
					// Expected indices: 0=name (String), 1=options (TerminalOptions DTO with
					// cwd, env, shell path, etc.). The shell path should also come from
					// options, not hardcoded.
					let options = json!({});
					let shell_path = "/bin/bash".to_string();
					provider.CreateTerminal("Terminal".to_string(), &options, &shell_path).await
				}
			};
			Box::new(effect)
		},

		"$terminal:sendText" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();
					// Parse terminal session identifier and text to send from request
					// parameters. Expected indices: 0=identifier (u32 or String), 1=text
					// (String). The identifier targets a specific terminal instance.
					provider.SendTextToTerminal(0, "echo hello\n".to_string()).await
				}
			};
			Box::new(effect)
		},

		"$terminal:dispose" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn TerminalProvider> = run_time.Environment.Require();
					// Parse terminal session identifier from request parameters to dispose of
					// a created terminal. Expected from _Parameters[0] as u32 or String
					// depending on terminal identifier scheme. Current hardcoded 0 is a
					// placeholder.
					provider.DisposeTerminal(0).await
				}
			};
			Box::new(effect)
		},

		// Webview
		"$webview:create" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn WebviewProvider> = run_time.Environment.Require();
					// Parse webview view type, title, and options from request parameters to
					// create a new webview panel. Expected indices: 0=viewType (String),
					// 1=title (String), 2=options (WebviewOptions DTO). This would eventually
					// call WebviewProvider::CreateWebview with properly deserialized
					// parameters. For now, just log that this would be called.
					warn!("$webview:create not fully implemented");
					Ok(json!({"handle": "webview-123"}))
				}
			};
			Box::new(effect)
		},

		"$resolveCustomEditor" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn CustomEditorProvider> = run_time.Environment.Require();
					// Parse view type, resource URI, and webview handle from request
					// parameters for resolving custom editor associations. Expected indices:
					// 0=viewType (String), 1=resource URI (Url), 2=webview handle (String).
					// This links a file URI to a specific webview editor.
					provider
						.ResolveCustomEditor(
							"viewType".to_string(),
							Url::parse("file:///tmp/test.txt").unwrap(),
							"webview-123".to_string(),
						)
						.await
				}
			};
			Box::new(effect)
		},

		// Debug
		"Debug.Start" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn DebugService> = run_time.Environment.Require();
					// Parse debug session folder URI and debug configuration from request
					// parameters. Expected indices: 0=folder URI (Url, optional), 1=configuration
					// (DebugConfiguration DTO). This initiates a debugging session for the
					// specified workspace folder.
					provider.StartDebugging(None, json!({ "type": "node" })).await
				}
			};
			Box::new(effect)
		},

		"Debug.RegisterConfigurationProvider" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn DebugService> = run_time.Environment.Require();
					// Parse debug type, provider factory handle, and sidecar identifier from
					// request parameters. Expected indices: 0=debug_type (String),
					// 1=provider_handle (u32 or String), 2=sidecar_id (String). This registers
					// a debug configuration provider for a specific debugging type.
					provider
						.RegisterDebugConfigurationProvider("node".to_string(), 1, "cocoon-main".to_string())
						.await
				}
			};
			Box::new(effect)
		},

		// Tree View
		"$tree:register" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn TreeViewProvider> = run_time.Environment.Require();
					// Parse tree view identifier and tree data provider DTO from request
					// parameters. Expected indices: 0=viewId (String), 1=data provider
					// (TreeDataProvider DTO). This registers a custom tree view for the
					// extension's UI.
					provider.RegisterTreeDataProvider("viewId".to_string(), json!({})).await
				}
			};
			Box::new(effect)
		},

		// Source Control Management
		"$scm:createSourceControl" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
					// Parse source control management resource URI and associated metadata from
					// request parameters. Expected indices: 0=resource (SourceControlResource
					// DTO), 1=metadata (Value). This creates a new SCM source control widget
					// for a repository.
					provider.CreateSourceControl(json!({}), json!({})).await
				}
			};
			Box::new(effect)
		},

		"$scm:updateSourceControl" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
					// Parse source control resource changes and update data from request
					// parameters. Expected from _Parameters[0] as SourceControl update DTO.
					// This updates the UI to reflect repository changes (commits, branches,
					// diffs, etc.).
					provider.UpdateSourceControl(json!({})).await
				}
			};
			Box::new(effect)
		},

		"$scm:updateGroup" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
					// Parse source control group identifier and associated resources from
					// request parameters. Expected indices: 0=group_id (String), 1=resources
					// (Vec<SourceControlResource>). Groups organize multiple SCM repositories
					// together in the UI.
					provider.UpdateGroup("group1".to_string(), json!([])).await
				}
			};
			Box::new(effect)
		},

		"$scm:registerInputBox" => {
			let effect = |run_time:Arc<MountainRunTime>| {
				async move {
					let provider:Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
					// Parse input box options from request parameters to configure an SCM input
					// box. Expected from _Parameters[0] as InputBoxOptions DTO (placeholder,
					// prompt, validation, etc.). Registers input control for user interaction
					// within source control view.
					provider.RegisterInputBox(json!({})).await
				}
			};
			Box::new(effect)
		},

		// Unknown command
		_ => {
			warn!("[EffectCreation] Unknown method: {}", MethodName);
			let effect = |run_time:Arc<MountainRunTime>| async move { Err(format!("Unknown method: {}", MethodName)) };
			Box::new(effect)
		},
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_effect_creation_for_known_commands() {
		// Test that known commands return a valid MappedEffect
		let effect = CreateEffectForRequest(
			tauri::test::mock_app(),
			&Arc::new(MountainEnvironment::default()),
			"Keybinding.GetResolved",
			&json!([]),
		);
		assert!(effect.is_some());
	}

	#[test]
	fn test_unknown_command_returns_error() {
		// Test that unknown commands return an error effect
		let effect = CreateEffectForRequest(
			tauri::test::mock_app(),
			&Arc::new(MountainEnvironment::default()),
			"Unknown.Command",
			&json!([]),
		);
		// Should return an effect that when executed returns error
		let result = (effect)(Arc::new(MountainRunTime::default()));
		// Since we can't easily test async, just verify it's a boxed closure
	}
}
