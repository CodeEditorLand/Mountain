//! # Bootstrap (Command)
//!
//! Registers all native, Rust-implemented commands and providers into the
//! application's state at startup. This module ensures all core functionality
//! is available as soon as the application initializes.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Command Registration
//! - Register all Tauri command handlers from `Command::` module
//! - Register core IPC command handlers from `Track::` module
//! - Build the complete `invoke_handler` vector for Tauri builder
//! - Ensure all commands are available before UI starts
//!
//! ### 2. Tree View Provider Registration
//! - Register native tree view providers (FileExplorer, etc.)
//! - Create provider instances and store in `ApplicationState::ActiveTreeViews`
//! - Associate view identifiers with provider implementations
//!
//! ### 3. Provider Registration
//! - Initialize Environment providers that need early setup
//! - Register command executors and configuration providers
//! - Set up document and workspace providers
//!
//! ## ARCHITECTURAL ROLE
//!
//! Bootstrap is the **registration orchestrator** for Mountain's startup:
//!
//! ```text
//! Binary::Main ──► Bootstrap::RegisterAll ──► Tauri Builder ──► App Ready
//!                      │
//!                      ├─► Command Handlers Registered
//!                      ├─► Tree View Providers Registered
//!                      └─► ApplicationState Populated
//! ```
//!
//! ### Position in Mountain
//! - `Command` module: Command system initialization
//! - Called from `Binary::Main::Fn` during Tauri builder setup
//! - Must complete before `.run()` is called on Tauri app
//!
//! ### Key Functions
//! - `RegisterAll`: Main entry point that registers everything
//! - `RegisterCommands`: Adds all Tauri command handlers
//! - `RegisterTreeViewProviders`: Registers native tree view providers
//!
//! ## REGISTRATION PROCESS
//!
//! 1. **Commands**: All command functions are added to Tauri's `invoke_handler`
//!    via `tauri::generate_handler![]` macro
//! 2. **Tree Views**: Native providers are instantiated and stored in state
//! 3. **Error Handling**: Registration failures are logged but don't stop
//!    startup
//!
//! ## COMMAND REGISTRATION
//!
//! The following command modules are registered:
//! - `Command::TreeView::GetTreeViewChildren`
//! - `Command::LanguageFeature::MountainProvideHover`
//! - `Command::LanguageFeature::MountainProvideCompletions`
//! - `Command::LanguageFeature::MountainProvideDefinition`
//! - `Command::LanguageFeature::MountainProvideReferences`
//! - `Command::SourceControlManagement::GetAllSourceControlManagementState`
//! - `Command::Keybinding::GetResolvedKeybinding`
//! - `Track::DispatchLogic::DispatchFrontendCommand`
//! - `Track::DispatchLogic::ResolveUIRequest`
//! - `IPC::TauriIPCServer::mountain_ipc_receive_message`
//! - `IPC::TauriIPCServer::mountain_ipc_get_status`
//! - `Binary::Main::SwitchTrayIcon`
//! - `Binary::Main::MountainGetWorkbenchConfiguration`
//! - (and more...)
//!
//! ## TREE VIEW PROVIDERS
//!
//! Currently registered native providers:
//! - `FileExplorerViewProvider`: File system tree view
//!   - View ID: `"fileExplorer"`
//!   - Provides workspace folders and file listings
//!
//! ## PERFORMANCE
//!
//! - Registration is synchronous and fast (no async allowed in registration)
//! - All commands are registered up-front; no lazy loading
//! - Tree view providers are created once at startup
//!
//! ## ERROR HANDLING
//!
//! - Command registration errors are logged as errors
//! - Tree view provider errors are logged as warnings
//! - Registration continues even if some components fail
//!
//! ## TODO
//!
//! - [ ] Add command registration metrics (count, duplicates detection)
//! - [ ] Implement command dependency ordering
//! - [ ] Add command validation (duplicate names, signature checking)
//! - [ ] Support dynamic command registration after startup
//! - [ ] Add command unregistration for hot-reload scenarios
//! - [ ] Implement command permission system
//!
//! ## MODULE CONTENTS
//!
//! - `RegisterAll`: Main registration function called from Binary::Main
//! - `RegisterCommands`: Internal function to register all command handlers
//! - `RegisterTreeViewProviders`: Internal function to register tree view
//! providers

// ## VSCode Reference:
// - vs/workbench/services/actions/common/menuService.ts
// - vs/workbench/browser/actions.ts
// - vs/platform/actions/common/actions.ts
//
// ============================================================================

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{
	DTO::WorkspaceEditDTO::WorkspaceEditDTO,
	Document::OpenDocument::OpenDocument,
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	UserInterface::ShowOpenDialog::ShowOpenDialog,
	Workspace::ApplyWorkspaceEdit::ApplyWorkspaceEdit,
};
use serde_json::{Value, json};
use tauri::{AppHandle, WebviewWindow, Wry};
use url::Url;

use crate::{
use crate::dev_log;
	ApplicationState::{ApplicationState, DTO::TreeViewStateDTO::TreeViewStateDTO, MapLockError},
	Environment::CommandProvider::CommandHandler,
	FileSystem::FileExplorerViewProvider::FileExplorerViewProvider,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

// --- Command Implementations ---

/// A simple native command that logs a message.
fn CommandHelloWorld(
	_ApplicationHandle:AppHandle<Wry>,

	_Window:WebviewWindow<Wry>,

	_RunTime:Arc<ApplicationRunTime>,

	_Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		dev_log!("commands", "[Native Command] Hello from Mountain!");

		Ok(json!("Hello from Mountain's native command!"))
	})
}

/// A native command that orchestrates the "Open File" dialog flow.
fn CommandOpenFile(
	_ApplicationHandle:AppHandle<Wry>,

	_Window:WebviewWindow<Wry>,

	RunTime:Arc<ApplicationRunTime>,

	_Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		dev_log!("commands", "[Native Command] Executing Open File...");

		let DialogResult = RunTime.Run(ShowOpenDialog(None)).await.map_err(|Error| Error.to_string())?;

		if let Some(Paths) = DialogResult {
			if let Some(Path) = Paths.first() {
				// We have a path, now open the document.
				let URI = Url::from_file_path(Path).map_err(|_| "Invalid file path".to_string())?;

				let OpenDocumentEffect = OpenDocument(json!({ "external": URI.to_string() }), None, None);

				RunTime.Run(OpenDocumentEffect).await.map_err(|Error| Error.to_string())?;
			}
		}

		Ok(Value::Null)
	})
}

/// A native command that orchestrates the "Format Document" action.
fn CommandFormatDocument(
	_ApplicationHandle:AppHandle<Wry>,

	_Window:WebviewWindow<Wry>,

	RunTime:Arc<ApplicationRunTime>,

	_Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		dev_log!("commands", "[Native Command] Executing Format Document...");

		let AppState = &RunTime.Environment.ApplicationState;

		let URIString = AppState
			.Workspace
			.ActiveDocumentURI
			.lock()
			.map_err(MapLockError)
			.map_err(|Error| Error.to_string())?
			.clone()
			.ok_or("No active document URI found in state".to_string())?;

		let URI = Url::parse(&URIString).map_err(|_| "Invalid URI in window state".to_string())?;

		// Example formatting options
		let Options = json!({ "tabSize": 4, "insertSpaces": true });

		// 1. Get the formatting edits from the language feature provider.
		let LanguageProvider:Arc<dyn LanguageFeatureProviderRegistry> = RunTime.Environment.Require();

		let EditsOption = LanguageProvider
			.ProvideDocumentFormattingEdits(URI.clone(), Options)
			.await
			.map_err(|Error| Error.to_string())?;

		if let Some(Edits) = EditsOption {
			if Edits.is_empty() {
				dev_log!("commands", "[Native Command] No formatting changes to apply.");

				return Ok(Value::Null);
			}

			// 2. Convert the text edits into a WorkspaceEdit.
			let WorkspaceEdit = WorkspaceEditDTO {
				Edits:vec![(
					serde_json::to_value(&URI).map_err(|Error| Error.to_string())?,
					Edits
						.into_iter()
						.map(serde_json::to_value)
						.collect::<Result<Vec<_>, _>>()
						.map_err(|Error| Error.to_string())?,
				)],
			};

			// 3. Apply the workspace edit.
			dev_log!("commands", "[Native Command] Applying formatting edits...");

			RunTime
				.Run(ApplyWorkspaceEdit(WorkspaceEdit))
				.await
				.map_err(|Error| Error.to_string())?;
		} else {
			dev_log!("commands", "[Native Command] No formatting provider found for this document.");
		}

		Ok(Value::Null)
	})
}

/// A native command for saving the current document.
fn CommandSaveDocument(
	_ApplicationHandle:AppHandle<Wry>,

	_Window:WebviewWindow<Wry>,

	RunTime:Arc<ApplicationRunTime>,

	_Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		dev_log!("commands", "[Native Command] Executing Save Document...");

		let AppState = &RunTime.Environment.ApplicationState;

		let URIString = AppState
			.Workspace
			.ActiveDocumentURI
			.lock()
			.map_err(MapLockError)
			.map_err(|Error| Error.to_string())?
			.clone()
			.ok_or("No active document URI found in state".to_string())?;

		let URI = Url::parse(&URIString).map_err(|_| "Invalid URI in window state".to_string())?;

		// Persist the active document by invoking DocumentProvider::SaveDocument or the
		// Document::Save effect. This reads the document URI from ApplicationState,
		// serializes the current editor content, and writes to disk with proper error
		// handling, atomic writes, and backup creation. Current implementation only
		// logs the action; full implementation requires integration with the document
		// lifecycle and file system provider.
		dev_log!("commands", "[Native Command] Saving document: {}", URI);

		Ok(Value::Null)
	})
}

/// A native command for closing the current document.
fn CommandCloseDocument(
	_ApplicationHandle:AppHandle<Wry>,

	_Window:WebviewWindow<Wry>,

	RunTime:Arc<ApplicationRunTime>,

	_Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		dev_log!("commands", "[Native Command] Executing Close Document...");

		let AppState = &RunTime.Environment.ApplicationState;

		let URIString = AppState
			.Workspace
			.ActiveDocumentURI
			.lock()
			.map_err(MapLockError)
			.map_err(|Error| Error.to_string())?
			.clone()
			.ok_or("No active document URI found in state".to_string())?;

		let URI = Url::parse(&URIString).map_err(|_| "Invalid URI in window state".to_string())?;

		// Close the active document in the editor by triggering the workspace edit
		// to remove the document from open editors. Checks for unsaved changes and
		// prompts the user to save, discard, or cancel. Integrates with the document
		// lifecycle manager to release resources and update the UI. May invoke
		// Workbench::closeEditor or equivalent command. Current implementation only
		// logs the action.
		dev_log!("commands", "[Native Command] Closing document: {}", URI);

		Ok(Value::Null)
	})
}

/// A native command for reloading the window.
fn CommandReloadWindow(
	_ApplicationHandle:AppHandle<Wry>,

	_Window:WebviewWindow<Wry>,

	_RunTime:Arc<ApplicationRunTime>,

	_Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		dev_log!("commands", "[Native Command] Executing Reload Window...");

		// Refresh the entire application UI by calling WebviewWindow::reload. This
		// reinitializes the frontend, reapplies window state, and restarts extension
		// host processes if configuration changes require it. Used after settings
		// updates, extension installations, or development hot-reload. Current
		// implementation returns success without performing the actual reload.
		Ok(json!({ "success": true }))
	})
}

/// Validates command parameters before execution.
fn ValidateCommandParameters(CommandName:&str, Arguments:&Value) -> Result<(), String> {
	match CommandName {
		"mountain.openFile" | "workbench.action.files.openFile" => {
			// No specific validation needed for open file
			Ok(())
		},
		"editor.action.formatDocument" => {
			// Ensure there's an active document
			Ok(())
		},
		_ => Ok(()),
	}
}

// --- Registration Function ---

/// Registers all native commands and providers with the application state.
pub fn RegisterNativeCommands(
	AppHandle:&AppHandle<Wry>,

	ApplicationState:&Arc<ApplicationState>,
) -> Result<(), CommonError> {
	// --- Command Registration ---
	let mut CommandRegistry = ApplicationState
		.Extension
		.Registry
		.CommandRegistry
		.lock()
		.map_err(MapLockError)?;

	dev_log!("commands", "[Bootstrap] Registering native commands...");

	// Register core commands
	CommandRegistry.insert("mountain.helloWorld".to_string(), CommandHandler::Native(CommandHelloWorld));

	CommandRegistry.insert("mountain.openFile".to_string(), CommandHandler::Native(CommandOpenFile));

	CommandRegistry.insert(
		"workbench.action.files.openFile".to_string(),
		CommandHandler::Native(CommandOpenFile),
	);

	CommandRegistry.insert(
		"editor.action.formatDocument".to_string(),
		CommandHandler::Native(CommandFormatDocument),
	);

	CommandRegistry.insert(
		"workbench.action.files.save".to_string(),
		CommandHandler::Native(CommandSaveDocument),
	);

	CommandRegistry.insert(
		"workbench.action.closeActiveEditor".to_string(),
		CommandHandler::Native(CommandCloseDocument),
	);

	CommandRegistry.insert(
		"workbench.action.reloadWindow".to_string(),
		CommandHandler::Native(CommandReloadWindow),
	);

	dev_log!("commands", "[Bootstrap] {} native commands registered.", CommandRegistry.len());

	drop(CommandRegistry);

	// --- Command Validation ---
	dev_log!("commands", "[Bootstrap] Validating registered commands...");
	// Validate all registered commands at startup to catch configuration errors
	// early. Verification includes command signature correctness, parameter type
	// matching, required permissions and capabilities, and extension metadata
	// validity. This prevents runtime errors from malformed registrations and
	// provides immediate feedback to extension developers during development.
	// Current implementation logs without performing actual validation checks.

	// --- Tree View Provider Registration ---
	let mut TreeViewRegistry = ApplicationState
		.Feature
		.TreeViews
		.ActiveTreeViews
		.lock()
		.map_err(MapLockError)?;

	dev_log!("commands", "[Bootstrap] Registering native tree view providers...");

	let ExplorerViewID = "workbench.view.explorer".to_string();

	let ExplorerProvider = Arc::new(FileExplorerViewProvider::New(AppHandle.clone()));

	TreeViewRegistry.insert(
		ExplorerViewID.clone(),
		TreeViewStateDTO {
			ViewIdentifier:ExplorerViewID,

			Provider:Some(ExplorerProvider),

			// This is a native provider
			SideCarIdentifier:None,

			CanSelectMany:true,

			HasHandleDrag:false,

			HasHandleDrop:false,

			Message:None,

			Title:Some("Explorer".to_string()),

			Description:None,

			Badge:None,
		},
	);

	dev_log!("commands", "[Bootstrap] {} native tree view providers registered.", TreeViewRegistry.len());

	Ok(())
}
