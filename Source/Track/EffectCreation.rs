// File: Mountain/Source/Track/EffectCreation.rs
//
// # Central Routing Table for Mountain Application
//
// ## Responsibilities
//
// This module serves as the **central routing table** for the entire Mountain
// backend. Its primary responsibilities are:
//
// ### 1. Command-to-Effect Mapping
// - Map string-based command and RPC method names to their strongly-typed
//   effect constructors from the `Common` crate
// - Ensure all commands from Wind (frontend) and Cocoon (sidecar) have
//   corresponding effects
// - Provide default/error handling for unknown commands
// - Create a runnable, type-erased `MappedEffect` for each request
//
// ### 2. Effect System Integration
// - Transform declarative ActionEffects into executable closures
// - Ensure proper parameter deserialization and validation
// - Handle type-safe effect creation with comprehensive error context
// - Support both direct provider calls and effect-based execution
//
// ### 3. Error Handling and Validation
// - Validate all input parameters before effect creation
// - Provide detailed error messages for failed mappings
// - Handle serialization/deserialization errors gracefully
// - Support parameter defaults and optionals
//
// ## Architectural Role
//
// EffectCreation is the **routing layer** that sits between:
//
// DispatchLogic (Router) ──► EffectCreation (Mapper) ──► ApplicationRunTime
// (Executor)
//
// ### Design Patterns:
// 1. **Command Pattern**: Each command is mapped to a specific effect
// 2. **Factory Pattern**: The CreateEffectForRequest function creates effects
// 3. **Strategy Pattern**: Direct provider calls vs effect-based execution
//
// ### VS Code Reference:
// This module borrows from VS Code's command registration and dispatch
// system in `vs/workbench/services/extensions/common/extensions.ts`
// and `vs/platform/commands/common/commands.ts`. Key concepts:
// - Command ID → Handler mapping
// - Type-safe parameter passing
// - Async execution with proper error propagation
//
// ## Key Components
//
// ### CreateEffectForRequest
// The primary entry point that maps method names to effects. Returns a
// `MappedEffect` which is a boxed, async closure that can be executed by
// the ApplicationRunTime.
//
// ### Direct Provider Calls
// For performance-critical operations, we bypass the effect system and call
// providers directly. This is done for:
// - Configuration inspection/updates (high frequency)
// - Diagnostics (real-time updates)
// - Language features (interactive editing)
// - Terminal operations (direct user input)
//
// ### Effect-Based Handlers
// Most operations go through the effect system for:
// - Consistency and maintainability
// - Declarative semantics
// - Easier testing and mocking
// - Better error handling
//
// ## Supported Command Categories
//
// ### Commands
// - `Command.Execute` - Execute a registered command
// - `Command.GetAll` - Get all available commands
// - `Command.Register` - Register a new command
//
// ### Configuration
// - `Configuration.Inspect` - Inspect a configuration value
// - `Configuration.Update` - Update a configuration value
// - `Configuration.Get` - Get configuration sections
//
// ### Documents
// - `Document.Save` - Save a document
// - `Document.SaveAs` - Save a document to a new location
//
// ### FileSystem
// - `FileSystem.ReadFile` - Read file contents
// - `FileSystem.WriteFile` - Write file contents
// - `FileSystem.ReadDirectory` - List directory contents
// - `FileSystem.StatFile` - Get file metadata
// - `FileSystem.Delete` - Delete files/directories
//
// ### Debug
// - `Debug.Start` - Start a debugging session
// - `Debug.RegisterConfigurationProvider` - Register debug config provider
//
// ### Diagnostics
// - `Diagnostic.Set` - Set diagnostics for a resource
// - `Diagnostic.Clear` - Clear diagnostics
//
// ### Keybinding
// - `Keybinding.GetResolved` - Get resolved keybindings
//
// ### Language Features
// - `$languageFeatures:registerProvider` - Register a language feature provider
// - `$languageFeatures:unregisterProvider` - Unregister a provider
//
// ### Search
// - `Search.TextSearch` - Perform text search
//
// ### Source Control Management
// - `$scm:createSourceControl` - Create SCM provider
// - `$scm:updateSourceControl` - Update SCM state
// - `$scm:updateGroup` - Update SCM resource groups
// - `$scm:registerInputBox` - Register SCM input box
//
// ### Status Bar
// - `$statusBar:set` - Set status bar entry
// - `$statusBar:dispose` - Dispose status bar entry
// - `$setStatusBarMessage` - Set status bar message
// - `$disposeStatusBarMessage` - Dispose status bar message
//
// ### Storage
// - `Storage.Get` - Get a storage item
// - `Storage.Set` - Set a storage item
// - `$storage:getAll` - Get all storage items
// - `$storage:setAll` - Set all storage items
//
// ### Terminal
// - `$terminal:create` - Create a terminal instance
// - `$terminal:sendText` - Send text to terminal
// - `$terminal:dispose` - Dispose a terminal
//
// ### Tree View
// - `$tree:register` - Register a tree data provider
//
// ### User Interface
// - `UserInterface.ShowMessage` - Show a message dialog
// - `UserInterface.ShowOpenDialog` - Show open file dialog
// - `UserInterface.ShowSaveDialog` - Show save file dialog
//
// ### WebView
// - `$webview:create` - Create a webview panel
// - `$resolveCustomEditor` - Resolve a custom editor
//
// ## Error Handling
//
// All effects return `Result<Value, String>` where:
// - `Ok(Value)` - Successful execution with JSON-serializable result
// - `Err(String)` - Error with descriptive message
//
// Error recovery mechanisms:
// - Invalid parameters return descriptive errors
// - Unknown commands are caught and reported
// - Serialization errors are caught and reported
// - Provider call errors are propagated with context
//
// ## TODOs
//
// High Priority:
// - [ ] Add command parameter schema validation
// - [ ] Implement command permission checking
// - [ ] Add command deprecation warnings
//
// Medium Priority:
// - [ ] Cache frequently created effects
// - [ ] Add command timeout configuration
// - [ ] Implement command rate limiting
//
// Low Priority:
// - [ ] Add command metrics collection
// - [ ] Implement command aliasing
// - [ ] Add command migration support

//! # EffectCreation
//!
//! Contains the logic for creating `ActionEffect`s by mapping string-based
//! command and RPC method names to their strongly-typed effect constructors in
//! the `Common` crate. This is the central routing table of the application.

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
	WebView::WebViewProvider::WebViewProvider,
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

/// A helper that takes a specific `ActionEffect`, boxes it, and returns a
/// closure that can be run by the dispatcher. This is used for effects that
/// follow the declarative pattern.
fn Map<C, O, E>(Effect:ActionEffect<Arc<C>, E, O>) -> MappedEffect
where
	C: ?Sized + Send + Sync + 'static,
	O: serde::Serialize + Send + Sync + 'static,
	E: Into<CommonError> + From<CommonError> + Send + Sync + 'static,
	MountainEnvironment: Requires<C>, {
	Box::new(move |RunTime:Arc<MountainRunTime>| {
		Box::pin(async move {
			let Result = RunTime.Run(Effect).await;

			match Result {
				Ok(Output) => serde_json::to_value(Output).map_err(|Error| format!("Serialization failed: {}", Error)),

				Err(Error) => {
					let CommonError:CommonError = Error.into();

					Err(CommonError.to_string())
				},
			}
		})
	})
}

/// Creates an `ActionEffect` or a direct provider call for a request from any
/// source (frontend or sidecar). This function is the primary router for the
/// entire backend application logic.
pub fn CreateEffectForRequest<R:Runtime>(
	_ApplicationHandle:&AppHandle<R>,

	Method:&str,

	Parameters:Value,
) -> Result<MappedEffect, String> {
	let ParametersArray = Parameters
		.as_array()
		.ok_or_else(|| format!("Parameters for '{}' must be an array.", Method))?;

	// --- Direct Provider Calls (for performance or simplicity) ---
	// These bypass the `box_effect` helper for direct invocation.
	match Method {
		// Configuration
		"Configuration.Inspect" => {
			let Key = Parameter!(ParametersArray, 0, String)?;

			let Overrides = ParametersArray.get(1).cloned().unwrap_or(Value::Null);

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn ConfigurationInspector> = runtime.Environment.Require();

					let result = provider
						.InspectConfigurationValue(Key, from_value(Overrides).unwrap_or_default())
						.await
						.map_err(|Error| Error.to_string())?;

					Ok(serde_json::to_value(result).unwrap_or(Value::Null))
				})
			}));
		},

		"Configuration.Update" => {
			let Key = Parameter!(ParametersArray, 0, String)?;

			let ValueToSet = ParametersArray.get(1).cloned().unwrap_or(Value::Null);

			let Target = Parameter!(ParametersArray, 2, ConfigurationTarget)?;

			let Overrides = ParametersArray.get(3).cloned().unwrap_or(Value::Null);

			let ScopeToLang = Parameter!(ParametersArray, 4, Option<bool>)?;

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn ConfigurationProvider> = runtime.Environment.Require();

					provider
						.UpdateConfigurationValue(
							Key,
							ValueToSet,
							Target,
							from_value(Overrides).unwrap_or_default(),
							ScopeToLang,
						)
						.await
						.map_err(|Error| Error.to_string())?;

					Ok(Value::Null)
				})
			}));
		},

		// Custom Editor
		"$resolveCustomEditor" => {
			let ViewType = Parameter!(ParametersArray, 0, String)?;

			let ResourceURI = Parameter!(ParametersArray, 1, Url)?;

			let WebViewPanelHandle = Parameter!(ParametersArray, 2, String)?;

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn CustomEditorProvider> = runtime.Environment.Require();

					provider
						.ResolveCustomEditor(ViewType, ResourceURI, WebViewPanelHandle)
						.await
						.map_err(|Error| Error.to_string())?;

					Ok(Value::Null)
				})
			}));
		},

		// Debug
		"Debug.Start" => {
			let folder_uri_val = ParametersArray.get(0).cloned().unwrap_or(Value::Null);

			let config = ParametersArray.get(1).cloned().unwrap_or(Value::Null);

			let folder_uri = if folder_uri_val.is_null() {
				None
			} else {
				Some(from_value(folder_uri_val).map_err(|Error| Error.to_string())?)
			};

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn DebugService> = runtime.Environment.Require();

					let session_id = provider
						.StartDebugging(folder_uri, config)
						.await
						.map_err(|Error| Error.to_string())?;

					Ok(json!(session_id))
				})
			}));
		},

		"Debug.RegisterConfigurationProvider" => {
			let dbg_type = Parameter!(ParametersArray, 0, String)?;

			let handle = Parameter!(ParametersArray, 1, u32)?;

			let sidecar_id = Parameter!(ParametersArray, 2, String)?;

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn DebugService> = runtime.Environment.Require();

					provider
						.RegisterDebugConfigurationProvider(dbg_type, handle, sidecar_id)
						.await
						.map_err(|Error| Error.to_string())?;

					Ok(Value::Null)
				})
			}));
		},

		// Diagnostic
		"Diagnostic.Set" => {
			let Owner = Parameter!(ParametersArray, 0, String)?;

			let Entries = ParametersArray.get(1).cloned().unwrap_or(Value::Null);

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn DiagnosticManager> = runtime.Environment.Require();

					provider
						.SetDiagnostics(Owner, Entries)
						.await
						.map_err(|Error| Error.to_string())?;

					Ok(Value::Null)
				})
			}));
		},

		"Diagnostic.Clear" => {
			let Owner = Parameter!(ParametersArray, 0, String)?;

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn DiagnosticManager> = runtime.Environment.Require();

					provider.ClearDiagnostics(Owner).await.map_err(|Error| Error.to_string())?;

					Ok(Value::Null)
				})
			}));
		},

		// Keybinding
		"Keybinding.GetResolved" => {
			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn KeybindingProvider> = runtime.Environment.Require();

					provider.GetResolvedKeybinding().await.map_err(|Error| Error.to_string())
				})
			}));
		},

		// Language Features
		"$languageFeatures:registerProvider" => {
			let SideCarID = Parameter!(ParametersArray, 0, String)?;

			let ProviderType = Parameter!(ParametersArray, 1, ProviderType)?;

			let Selector = ParametersArray.get(2).cloned().unwrap_or(Value::Null);

			let ExtensionID = ParametersArray.get(3).cloned().unwrap_or(Value::Null);

			let Options = ParametersArray.get(4).cloned();

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn LanguageFeatureProviderRegistry> = runtime.Environment.Require();

					let handle = provider
						.RegisterProvider(SideCarID, ProviderType, Selector, ExtensionID, Options)
						.await
						.map_err(|Error| Error.to_string())?;

					Ok(json!(handle))
				})
			}));
		},

		"$languageFeatures:unregisterProvider" => {
			let Handle = Parameter!(ParametersArray, 0, u32)?;

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn LanguageFeatureProviderRegistry> = runtime.Environment.Require();

					provider.UnregisterProvider(Handle).await.map_err(|Error| Error.to_string())?;

					Ok(Value::Null)
				})
			}));
		},

		// Search
		"Search.TextSearch" => {
			let Query = ParametersArray.get(0).cloned().unwrap_or(Value::Null);

			let Options = ParametersArray.get(1).cloned().unwrap_or(Value::Null);

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn SearchProvider> = runtime.Environment.Require();

					let result = provider.TextSearch(Query, Options).await.map_err(|Error| Error.to_string())?;

					Ok(result)
				})
			}));
		},

		// SourceControlManagement
		"$scm:createSourceControl" => {
			let DTO = ParametersArray.get(0).cloned().unwrap_or(Value::Null);

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn SourceControlManagementProvider> = runtime.Environment.Require();

					let handle = provider.CreateSourceControl(DTO).await.map_err(|Error| Error.to_string())?;

					Ok(json!(handle))
				})
			}));
		},

		"$scm:updateSourceControl" => {
			let handle = Parameter!(ParametersArray, 0, u32)?;

			let dto = ParametersArray.get(1).cloned().unwrap_or(Value::Null);

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn SourceControlManagementProvider> = runtime.Environment.Require();

					provider
						.UpdateSourceControl(handle, dto)
						.await
						.map_err(|Error| Error.to_string())?;

					Ok(Value::Null)
				})
			}));
		},

		"$scm:updateGroup" => {
			let handle = Parameter!(ParametersArray, 0, u32)?;

			let dto = ParametersArray.get(1).cloned().unwrap_or(Value::Null);

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn SourceControlManagementProvider> = runtime.Environment.Require();

					provider
						.UpdateSourceControlGroup(handle, dto)
						.await
						.map_err(|Error| Error.to_string())?;

					Ok(Value::Null)
				})
			}));
		},

		"$scm:registerInputBox" => {
			let handle = Parameter!(ParametersArray, 0, u32)?;

			let dto = ParametersArray.get(1).cloned().unwrap_or(Value::Null);

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn SourceControlManagementProvider> = runtime.Environment.Require();

					provider
						.RegisterInputBox(handle, dto)
						.await
						.map_err(|Error| Error.to_string())?;

					Ok(Value::Null)
				})
			}));
		},

		// Status Bar
		"$statusBar:set" => {
			let EntryDTO = Parameter!(ParametersArray, 0, StatusBarEntryDTO)?;

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn StatusBarProvider> = runtime.Environment.Require();

					provider.SetStatusBarEntry(EntryDTO).await.map_err(|Error| Error.to_string())?;

					Ok(Value::Null)
				})
			}));
		},

		"$statusBar:dispose" => {
			let EntryID = Parameter!(ParametersArray, 0, String)?;

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn StatusBarProvider> = runtime.Environment.Require();

					provider
						.DisposeStatusBarEntry(EntryID)
						.await
						.map_err(|Error| Error.to_string())?;

					Ok(Value::Null)
				})
			}));
		},

		"$setStatusBarMessage" => {
			let ID = Parameter!(ParametersArray, 0, String)?;

			let Text = Parameter!(ParametersArray, 1, String)?;

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn StatusBarProvider> = runtime.Environment.Require();

					provider
						.SetStatusBarMessage(ID, Text)
						.await
						.map_err(|Error| Error.to_string())?;

					Ok(Value::Null)
				})
			}));
		},

		"$disposeStatusBarMessage" => {
			let ID = Parameter!(ParametersArray, 0, String)?;

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn StatusBarProvider> = runtime.Environment.Require();

					provider.DisposeStatusBarMessage(ID).await.map_err(|Error| Error.to_string())?;

					Ok(Value::Null)
				})
			}));
		},

		// Storage (Batch)
		"$storage:getAll" => {
			let IsGlobal = Parameter!(ParametersArray, 0, bool)?;

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn StorageProvider> = runtime.Environment.Require();

					provider.GetAllStorage(IsGlobal).await.map_err(|Error| Error.to_string())
				})
			}));
		},

		"$storage:setAll" => {
			let IsGlobal = Parameter!(ParametersArray, 0, bool)?;

			let State = ParametersArray.get(1).cloned().unwrap_or(Value::Null);

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn StorageProvider> = runtime.Environment.Require();

					provider
						.SetAllStorage(IsGlobal, State)
						.await
						.map_err(|Error| Error.to_string())?;

					Ok(Value::Null)
				})
			}));
		},

		// Terminal
		"$terminal:create" => {
			let OptionsValue = ParametersArray.get(0).cloned().unwrap_or(Value::Null);

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn TerminalProvider> = runtime.Environment.Require();

					let result = provider.CreateTerminal(OptionsValue).await.map_err(|Error| Error.to_string())?;

					Ok(result)
				})
			}));
		},

		"$terminal:sendText" => {
			let TerminalId = Parameter!(ParametersArray, 0, u64)?;

			let Text = Parameter!(ParametersArray, 1, String)?;

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn TerminalProvider> = runtime.Environment.Require();

					provider
						.SendTextToTerminal(TerminalId, Text)
						.await
						.map_err(|Error| Error.to_string())?;

					Ok(Value::Null)
				})
			}));
		},

		"$terminal:dispose" => {
			let TerminalId = Parameter!(ParametersArray, 0, u64)?;

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn TerminalProvider> = runtime.Environment.Require();

					provider.DisposeTerminal(TerminalId).await.map_err(|Error| Error.to_string())?;

					Ok(Value::Null)
				})
			}));
		},

		// Tree View
		"$tree:register" => {
			let ID = Parameter!(ParametersArray, 0, String)?;

			let Options = ParametersArray.get(1).cloned().unwrap_or(Value::Null);

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn TreeViewProvider> = runtime.Environment.Require();

					provider
						.RegisterTreeDataProvider(ID, Options)
						.await
						.map_err(|Error| Error.to_string())?;

					Ok(Value::Null)
				})
			}));
		},

		// WebView
		"$webview:create" => {
			let ExtData = Parameter!(ParametersArray, 0, Value)?;

			let ViewType = Parameter!(ParametersArray, 1, String)?;

			let Title = Parameter!(ParametersArray, 2, String)?;

			let ShowOpts = Parameter!(ParametersArray, 3, Value)?;

			let PanelOpts = Parameter!(ParametersArray, 4, Value)?;

			let ContentOpts = Parameter!(ParametersArray, 5, Value)?;

			return Ok(Box::new(move |runtime:Arc<MountainRunTime>| {
				Box::pin(async move {
					let provider:Arc<dyn WebViewProvider> = runtime.Environment.Require();

					let handle = provider
						.CreateWebViewPanel(ExtData, ViewType, Title, ShowOpts, PanelOpts, ContentOpts)
						.await
						.map_err(|Error| Error.to_string())?;

					Ok(json!(handle))
				})
			}));
		},

		// Fall through to effect-based handlers
		_ => {},
	}

	// --- ActionEffect-based Handlers ---
	let effect = match Method {
		// Command
		"Command.Execute" => {
			let ID = Parameter!(ParametersArray, 0, String)?;

			let Args = ParametersArray.get(1).cloned().unwrap_or(Value::Null);

			Map(ExecuteCommand(ID, Args))
		},

		"Command.GetAll" => Map(GetAllCommands()),

		"Command.Register" => {
			let SideCarID = Parameter!(ParametersArray, 0, String)?;

			let CommandID = Parameter!(ParametersArray, 1, String)?;

			Map(RegisterCommand(SideCarID, CommandID))
		},

		// Configuration
		"Configuration.Get" => {
			let Section = Parameter!(ParametersArray, 0, Option<String>)?;

			let Overrides = ParametersArray.get(1).cloned().unwrap_or(Value::Null);

			Map(GetConfiguration(Section, Overrides))
		},

		// Document
		"Document.Save" => {
			let uri_str = Parameter!(ParametersArray, 0, String)?;

			let uri = Url::parse(&uri_str).map_err(|Error| format!("Invalid URI parameter: {}", Error))?;

			Map(SaveDocument(uri))
		},

		"Document.SaveAs" => {
			let original_uri_str = Parameter!(ParametersArray, 0, String)?;

			let original_uri =
				Url::parse(&original_uri_str).map_err(|Error| format!("Invalid URI parameter: {}", Error))?;

			Map(SaveDocumentAs(original_uri, None))
		},

		// FileSystem
		"FileSystem.ReadFile" => {
			let Path = Parameter!(ParametersArray, 0, _)?;

			Map(ReadFile(Path))
		},

		"FileSystem.StatFile" => {
			let Path = Parameter!(ParametersArray, 0, _)?;

			Map(StatFile(Path))
		},

		"FileSystem.ReadDirectory" => {
			let Path = Parameter!(ParametersArray, 0, _)?;

			Map(ReadDirectory(Path))
		},

		"FileSystem.WriteFile" => {
			let Path = Parameter!(ParametersArray, 0, _)?;

			let Content = Parameter!(ParametersArray, 1, Vec<u8>)?;

			let Create = Parameter!(ParametersArray, 2, bool)?;

			let Overwrite = Parameter!(ParametersArray, 3, bool)?;

			Map(WriteFileBytes(Path, Content, Create, Overwrite))
		},

		"FileSystem.Delete" => {
			let Path = Parameter!(ParametersArray, 0, _)?;

			let Recursive = Parameter!(ParametersArray, 1, bool)?;

			let UseTrash = Parameter!(ParametersArray, 2, bool)?;

			Map(Delete(Path, Recursive, UseTrash))
		},

		// Storage (Legacy)
		"Storage.Get" => {
			let TargetObject = ParametersArray.get(0).cloned().unwrap_or(Value::Null);

			Map(GetStorageItem(TargetObject))
		},

		"Storage.Set" => {
			let TargetObject = ParametersArray.get(0).cloned().unwrap_or(Value::Null);

			let ValueToSet = ParametersArray.get(1).cloned().unwrap_or(Value::Null);

			Map(SetStorageItem(TargetObject, ValueToSet))
		},

		// UserInterface
		"UserInterface.ShowMessage" => {
			let Severity = Parameter!(ParametersArray, 0, _)?;

			let Message = Parameter!(ParametersArray, 1, String)?;

			let Options = ParametersArray.get(2).cloned().unwrap_or(Value::Null);

			Map(ShowMessage(Severity, Message, Options))
		},

		"UserInterface.ShowOpenDialog" => {
			let Options = Parameter!(ParametersArray, 0, _)?;

			Map(ShowOpenDialog(Options))
		},

		"UserInterface.ShowSaveDialog" => {
			let Options = Parameter!(ParametersArray, 0, _)?;

			Map(ShowSaveDialog(Options))
		},

		_ => return Err(format!("No mapping found for method: {}", Method)),
	};

	Ok(effect)
}
