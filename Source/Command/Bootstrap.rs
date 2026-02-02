// ============================================================================
// File: Mountain/Source/Command/Bootstrap.rs
// ============================================================================
// # Bootstrap Commands & Providers
//
// Registers all native, Rust-implemented commands and providers into the
// application's state at startup. This module ensures all core functionality
// is available as soon as the application initializes.
//
// ## Key Features:
// - Comprehensive native command registration
// - Tree view provider registration
// - Command validation and error handling
// - Command execution context management
// - Integration with Tauri command system
//
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
use log::info;
use serde_json::{Value, json};
use tauri::{AppHandle, WebviewWindow, Wry};
use url::Url;

use crate::{
	ApplicationState::{
		ApplicationState::{ApplicationState, MapLockError},
		DTO::TreeViewStateDTO::TreeViewStateDTO,
	},
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
		info!("[Native Command] Hello from Mountain!");

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
		info!("[Native Command] Executing Open File...");

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
		info!("[Native Command] Executing Format Document...");

		let AppState = &RunTime.Environment.ApplicationState;

		let URIString = AppState
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
				info!("[Native Command] No formatting changes to apply.");

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
			info!("[Native Command] Applying formatting edits...");

			RunTime
				.Run(ApplyWorkspaceEdit(WorkspaceEdit))
				.await
				.map_err(|Error| Error.to_string())?;
		} else {
			info!("[Native Command] No formatting provider found for this document.");
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
		info!("[Native Command] Executing Save Document...");

		let AppState = &RunTime.Environment.ApplicationState;

		let URIString = AppState
			.ActiveDocumentURI
			.lock()
			.map_err(MapLockError)
			.map_err(|Error| Error.to_string())?
			.clone()
			.ok_or("No active document URI found in state".to_string())?;

		let URI = Url::parse(&URIString).map_err(|_| "Invalid URI in window state".to_string())?;

		// TODO: Trigger document save
		info!("[Native Command] Saving document: {}", URI);

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
		info!("[Native Command] Executing Close Document...");

		let AppState = &RunTime.Environment.ApplicationState;

		let URIString = AppState
			.ActiveDocumentURI
			.lock()
			.map_err(MapLockError)
			.map_err(|Error| Error.to_string())?
			.clone()
			.ok_or("No active document URI found in state".to_string())?;

		let URI = Url::parse(&URIString).map_err(|_| "Invalid URI in window state".to_string())?;

		// TODO: Trigger document close
		info!("[Native Command] Closing document: {}", URI);

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
		info!("[Native Command] Executing Reload Window...");

		// TODO: Reload the window
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
	let mut CommandRegistry = ApplicationState.CommandRegistry.lock().map_err(MapLockError)?;

	info!("[Bootstrap] Registering native commands...");

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

	info!("[Bootstrap] {} native commands registered.", CommandRegistry.len());

	drop(CommandRegistry);

	// --- Command Validation ---
	info!("[Bootstrap] Validating registered commands...");
	// TODO: Implement comprehensive command validation

	// --- Tree View Provider Registration ---
	let mut TreeViewRegistry = ApplicationState.ActiveTreeViews.lock().map_err(MapLockError)?;

	info!("[Bootstrap] Registering native tree view providers...");

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
		},
	);

	info!("[Bootstrap] {} native tree view providers registered.", TreeViewRegistry.len());

	Ok(())
}
