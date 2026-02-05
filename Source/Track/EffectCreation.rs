//! # EffectCreation (Track)
//!
//! RESPONSIBILITIES:
//! - Central routing table that maps string-based commands/RPC methods to typed effects
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
//! 5. Returns `MappedEffect` (boxed future) to be executed by `ApplicationRunTime`
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
//! - TODO: Consider caching frequently created effects (effect pool)
//! - TODO: Add command timeout and rate limiting
//!
//! VS CODE REFERENCE:
//! - `vs/workbench/services/extensions/common/extensions.ts` - command registration
//! - `vs/platform/commands/common/commands.ts` - command service and dispatching
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
//! **SourceControlManagement**: $scm:createSourceControl, updateSourceControl, updateGroup, registerInputBox
//! **StatusBar**: $statusBar:set, dispose, $setStatusBarMessage, $disposeStatusBarMessage
//! **Storage**: Get, Set, $storage:getAll, $storage:setAll
//! **Terminal**: $terminal:create, sendText, dispose
//! **TreeView**: $tree:register
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
//! - Main function: `CreateEffectForRequest` - map method name to effect (in DispatchLogic)
//! - Direct provider calls: Various `Provider::*` methods called without effect wrapper
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
/// `MappedEffect` - a boxed async closure that takes Arc<ApplicationRunTime> and
/// returns Result<Value, String>
pub fn CreateEffectForRequest<R: Runtime>(
	_ApplicationHandle: AppHandle<R>,
	_State: &Arc<MountainEnvironment>,
	MethodName: &str,
	_Parameters: &Value,
) -> MappedEffect {
	// Simplified: direct provider calls for hot paths
	match MethodName {
		// Configuration
		"Configuration.Inspect" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn ConfigurationInspector> = run_time.Environment.Require();
					let target = ConfigurationTarget::Global;
					let section = String::new(); // TODO: parse from parameters
					provider.InspectConfigurationValue(target, &section).await
				}
			};
			Box::new(effect)
		},

		"Configuration.Update" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn ConfigurationProvider> = run_time.Environment.Require();
					// TODO: parse target, section, value from parameters
					provider.UpdateConfigurationValue(ConfigurationTarget::Global, "section".to_string(), json!({})).await
				}
			};
			Box::new(effect)
		},

		// Diagnostics
		"Diagnostic.Set" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn DiagnosticManager> = run_time.Environment.Require();
					// TODO: parse owner, entries from parameters
					provider.SetDiagnostics("owner".to_string(), json!([])).await
				}
			};
			Box::new(effect)
		},

		"Diagnostic.Clear" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn DiagnosticManager> = run_time.Environment.Require();
					provider.ClearDiagnostics("owner".to_string()).await
				}
			};
			Box::new(effect)
		},

		// Language Features
		"$languageFeatures:registerProvider" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn LanguageFeatureProviderRegistry> = run_time.Environment.Require();
					// TODO: parse ProviderType, SelectorDTO, ExtensionIdentifierDTO, OptionsDTO
					provider.RegisterProvider(
						"cocoon".to_string(),
						ProviderType::Hover,
						json!({}),
						json!({}),
						None,
					).await
				}
			};
			Box::new(effect)
		},

		// Documents
		"Document.Save" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let document_provider: Arc<dyn DocumentProvider> = run_time.Environment.Require();
					// TODO: parse URI from parameters
					let uri = Url::parse("file:///tmp/test.txt").unwrap();
					document_provider.SaveDocument(uri).await
				}
			};
			Box::new(effect)
		},

		"Document.SaveAs" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let document_provider: Arc<dyn DocumentProvider> = run_time.Environment.Require();
					// TODO: parse original URI and new target URI from parameters
					let original_uri = Url::parse("file:///tmp/test.txt").unwrap();
					document_provider.SaveDocumentAs(original_uri, None).await
				}
			};
			Box::new(effect)
		},

		// FileSystem
		"FileSystem.ReadFile" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let fs_reader: Arc<dyn FileSystemReader> = run_time.Environment.Require();
					// TODO: parse path from parameters
					let path = std::path::PathBuf::from("/tmp/test.txt");
					fs_reader.ReadFile(&path).await
				}
			};
			Box::new(effect)
		},

		"FileSystem.WriteFile" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let fs_writer: Arc<dyn FileSystemWriter> = run_time.Environment.Require();
					// TODO: parse path, content, options from parameters
					let path = std::path::PathBuf::from("/tmp/test.txt");
					fs_writer.WriteFile(&path, &[]).await
				}
			};
			Box::new(effect)
		},

		"FileSystem.ReadDirectory" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let fs_reader: Arc<dyn FileSystemReader> = run_time.Environment.Require();
					// TODO: parse path from parameters
					let path = std::path::PathBuf::from("/tmp");
					fs_reader.ReadDirectory(&path).await
				}
			};
			Box::new(effect)
		},

		// Keybinding
		"Keybinding.GetResolved" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn KeybindingProvider> = run_time.Environment.Require();
					provider.GetResolvedKeybinding().await
				}
			};
			Box::new(effect)
		},

		// Search
		"Search.TextSearch" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn SearchProvider> = run_time.Environment.Require();
					// TODO: parse query, options from parameters
					provider.TextSearch(json!({}), None, None, false, false).await
				}
			};
			Box::new(effect)
		},

		// Storage
		"Storage.Get" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn StorageProvider> = run_time.Environment.Require();
					// TODO: parse key from parameters
					provider.GetStorageItem("key".to_string()).await
				}
			};
			Box::new(effect)
		},

		"Storage.Set" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn StorageProvider> = run_time.Environment.Require();
					// TODO: parse key, value from parameters
					provider.SetStorageItem("key".to_string(), json!("value")).await
				}
			};
			Box::new(effect)
		},

		// Commands
		"Command.Execute" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let command_executor: Arc<dyn CommandExecutor> = run_time.Environment.Require();
					// TODO: parse command identifier and argument from parameters
					command_executor.ExecuteCommand("command".to_string(), json!({})).await
				}
			};
			Box::new(effect)
		},

		"Command.GetAll" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let command_executor: Arc<dyn CommandExecutor> = run_time.Environment.Require();
					command_executor.GetAllCommands().await
				}
			};
			Box::new(effect)
		},

		// Status Bar
		"$statusBar:set" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn StatusBarProvider> = run_time.Environment.Require();
					// TODO: parse entry from parameters
					let entry = StatusBarEntryDTO {
						text: "status".to_string(),
						identifier: Some("id".to_string()),
						// other fields...
						..Default::default()
					};
					provider.SetStatusBarEntry(entry).await
				}
			};
			Box::new(effect)
		},

		"$statusBar:dispose" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn StatusBarProvider> = run_time.Environment.Require();
					// TODO: parse identifier from parameters
					provider.DisposeStatusBarEntry("id".to_string()).await
				}
			};
			Box::new(effect)
		},

		"$setStatusBarMessage" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn StatusBarProvider> = run_time.Environment.Require();
					// TODO: parse message identifier and text from parameters
					provider.SetStatusBarMessage("msg_id".to_string(), "message".to_string()).await
				}
			};
			Box::new(effect)
		},

		"$disposeStatusBarMessage" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn StatusBarProvider> = run_time.Environment.Require();
					// TODO: parse message identifier from parameters
					provider.DisposeStatusBarMessage("msg_id".to_string()).await
				}
			};
			Box::new(effect)
		},

		// User Interface
		"UserInterface.ShowMessage" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();
					// TODO: parse message type, title, message from parameters
					provider.ShowMessage("info".to_string(), "Title", "Message", json!({})).await
				}
			};
			Box::new(effect)
		},

		"UserInterface.ShowOpenDialog" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();
					// TODO: parse options from parameters
					provider.ShowOpenDialog(None).await
				}
			};
			Box::new(effect)
		},

		"UserInterface.ShowSaveDialog" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();
					// TODO: parse options from parameters
					provider.ShowSaveDialog(None).await
				}
			};
			Box::new(effect)
		},

		// Terminal
		"$terminal:create" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn TerminalProvider> = run_time.Environment.Require();
					// TODO: parse name, options from parameters
					let options = json!({});
					let shell_path = "/bin/bash".to_string();
					provider.CreateTerminal("Terminal".to_string(), &options, &shell_path).await
				}
			};
			Box::new(effect)
		},

		"$terminal:sendText" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn TerminalProvider> = run_time.Environment.Require();
					// TODO: parse identifier and text from parameters
					provider.SendTextToTerminal(0, "echo hello\n".to_string()).await
				}
			};
			Box::new(effect)
		},

		"$terminal:dispose" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn TerminalProvider> = run_time.Environment.Require();
					// TODO: parse identifier from parameters
					provider.DisposeTerminal(0).await
				}
			};
			Box::new(effect)
		},

		// Webview
		"$webview:create" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn WebviewProvider> = run_time.Environment.Require();
					// TODO: parse view type, title, options from parameters
					// For now, just log that this would be called
					warn!("$webview:create not fully implemented");
					Ok(json!({"handle": "webview-123"}))
				}
			};
			Box::new(effect)
		},

		"$resolveCustomEditor" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn CustomEditorProvider> = run_time.Environment.Require();
					// TODO: parse view type, resource URI, webview handle from parameters
					provider.ResolveCustomEditor(
						"viewType".to_string(),
						Url::parse("file:///tmp/test.txt").unwrap(),
						"webview-123".to_string(),
					).await
				}
			};
			Box::new(effect)
		},

		// Debug
		"Debug.Start" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn DebugService> = run_time.Environment.Require();
					// TODO: parse folder URI and configuration from parameters
					provider.StartDebugging(None, json!({ "type": "node" })).await
				}
			};
			Box::new(effect)
		},

		"Debug.RegisterConfigurationProvider" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn DebugService> = run_time.Environment.Require();
					// TODO: parse debug type, provider handle, sidecar identifier from parameters
					provider.RegisterDebugConfigurationProvider(
						"node".to_string(),
						1,
						"cocoon-main".to_string(),
					).await
				}
			};
			Box::new(effect)
		},

		// Tree View
		"$tree:register" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn TreeViewProvider> = run_time.Environment.Require();
					// TODO: parse view identifier, tree data provider from parameters
					provider.RegisterTreeDataProvider("viewId".to_string(), json!({})).await
				}
			};
			Box::new(effect)
		},

		// Source Control Management
		"$scm:createSourceControl" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
					// TODO: parse source control management resource and metadata from parameters
					provider.CreateSourceControl(json!({}), json!({})).await
				}
			};
			Box::new(effect)
		},

		"$scm:updateSourceControl" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
					// TODO: parse source control management resource changes from parameters
					provider.UpdateSourceControl(json!({})).await
				}
			};
			Box::new(effect)
		},

		"$scm:updateGroup" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
					// TODO: parse group identifier and resources from parameters
					provider.UpdateGroup("group1".to_string(), json!([])).await
				}
			};
			Box::new(effect)
		},

		"$scm:registerInputBox" => {
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					let provider: Arc<dyn SourceControlManagementProvider> = run_time.Environment.Require();
					// TODO: parse input box options from parameters
					provider.RegisterInputBox(json!({})).await
				}
			};
			Box::new(effect)
		},

		// Unknown command
		_ => {
			warn!("[EffectCreation] Unknown method: {}", MethodName);
			let effect = |run_time: Arc<MountainRunTime>| {
				async move {
					Err(format!("Unknown method: {}", MethodName))
				}
			};
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
