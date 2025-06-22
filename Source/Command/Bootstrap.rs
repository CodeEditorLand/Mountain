// File: Mountain/Source/Commands/Bootstrap.rs
// Role: Registers all native, Rust-implemented commands and providers at
// startup. Responsibilities:
//   - Centralize the registration of all built-in functionality.
//   - Populate the `CommandRegistry` with native command handlers.
//   - Populate the `ActiveTreeViews` registry with native tree data providers.

//! # Bootstrap Commands & Providers
//!
//! Registers all native, Rust-implemented commands and providers into the
//! application's state at startup.

#![allow(non_snake_case, non_camel_case_types)]

use std::{future::Future, pin::Pin, sync::Arc};

use Common::{
	DTO::WorkSpaceEditDTO::WorkSpaceEditDTO,
	Document::OpenDocument::OpenDocument,
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Environment::Requires::Requires,
	LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	UserInterface::ShowOpenDialog::ShowOpenDialog,
	WorkSpace::ApplyWorkSpaceEdit::ApplyWorkSpaceEdit,
};
use log::info;
use serde_json::{Value, json};
use tauri::{AppHandle, WebviewWindow};
use url::Url;

use crate::{
	ApplicationState::{ApplicationState::ApplicationState, DTO::TreeViewStateDTO::TreeViewStateDTO},
	Environment::CommandProvider::CommandHandler,
	FileSystem::FileExplorerViewProvider::FileExplorerViewProvider,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	// TEMPORARY DISABLE
	// Update::UpdateService,
};

// --- Command Implementations ---

/// A simple native command that logs a message.
fn CommandHelloWorld(
	_ApplicationHandle:AppHandle,
	_Window:WebviewWindow,
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
	_ApplicationHandle:AppHandle,
	_Window:WebviewWindow,
	RunTime:Arc<ApplicationRunTime>,
	_Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		info!("[Native Command] Executing Open File...");
		let DialogResult = RunTime.Run(ShowOpenDialog(None)).await.map_err(|e| e.to_string())?;

		if let Some(Paths) = DialogResult {
			if let Some(Path) = Paths.first() {
				// We have a path, now open the document.
				let Uri = Url::from_file_path(Path).map_err(|_| "Invalid file path".to_string())?;
				let OpenDocumentEffect = OpenDocument(json!({ "external": Uri.to_string() }), None, None);
				RunTime.Run(OpenDocumentEffect).await.map_err(|e| e.to_string())?;
			}
		}

		Ok(Value::Null)
	})
}

// TEMPORARY DISABLE
// /// A native command that triggers the application update check.
// fn CommandCheckForUpdates(
// 	ApplicationHandle:AppHandle,
// 	_Window:WebviewWindow,
// 	RunTime:Arc<ApplicationRunTime>,
// 	_Argument:Value,
// ) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
// 	Box::pin(async move {
// 		// The `true` here means we will notify the user even if there's no update.
// 		UpdateService::CheckForUpdates(ApplicationHandle, RunTime, true)
// 			.await
// 			.map_err(|e| e.to_string())?;
// 		Ok(Value::Null)
// 	})
// }

/// A native command that orchestrates the "Format Document" action.
fn CommandFormatDocument(
	_ApplicationHandle:AppHandle,
	// Window is unused now
	_Window:WebviewWindow,
	RunTime:Arc<ApplicationRunTime>,
	_Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		info!("[Native Command] Executing Format Document...");

		let AppState = &RunTime.Environment.ApplicationState;
		let UriString = AppState
			.ActiveDocumentURI
			.lock()
			.unwrap()
			.clone()
			.ok_or("No active document URI found in state".to_string())?;

		let Uri = Url::parse(&UriString).map_err(|_| "Invalid URI in window state".to_string())?;

		// Example formatting options
		let Options = json!({ "tabSize": 4, "insertSpaces": true });

		// 1. Get the formatting edits from the language feature provider.
		let LanguageProvider:Arc<dyn LanguageFeatureProviderRegistry> = RunTime.Environment.Require();
		let EditsOption = LanguageProvider
			.ProvideDocumentFormattingEdits(Uri.clone(), Options)
			.await
			.map_err(|e| e.to_string())?;

		if let Some(Edits) = EditsOption {
			if Edits.is_empty() {
				info!("[Native Command] No formatting changes to apply.");
				return Ok(Value::Null);
			}

			// 2. Convert the text edits into a WorkSpaceEdit.
			let WorkSpaceEdit = WorkSpaceEditDTO {
				Edits:vec![(
					serde_json::to_value(&Uri).unwrap(),
					Edits.into_iter().map(|e| serde_json::to_value(e).unwrap()).collect(),
				)],
			};

			// 3. Apply the workspace edit.
			info!("[Native Command] Applying formatting edits...");
			RunTime
				.Run(ApplyWorkSpaceEdit(WorkSpaceEdit))
				.await
				.map_err(|e| e.to_string())?;
		} else {
			info!("[Native Command] No formatting provider found for this document.");
		}

		Ok(Value::Null)
	})
}

// --- Registration Function ---

/// Registers all native commands and providers with the application state.
pub fn RegisterNativeCommands(AppHandle:&AppHandle, ApplicationState:&Arc<ApplicationState>) {
	// --- Command Registration ---
	let mut CommandRegistry = ApplicationState.CommandRegistry.lock().unwrap();
	info!("[Bootstrap] Registering native commands...");

	CommandRegistry.insert("mountain.helloWorld".to_string(), CommandHandler::Native(CommandHelloWorld));
	CommandRegistry.insert("mountain.openFile".to_string(), CommandHandler::Native(CommandOpenFile));
	CommandRegistry.insert(
		"workbench.action.files.openFile".to_string(),
		CommandHandler::Native(CommandOpenFile),
	);
	// TEMPORARY DISABLE
	// CommandRegistry.insert(
	// 	"mountain.checkForUpdates".to_string(),
	// 	CommandHandler::Native(CommandCheckForUpdates),
	// );
	CommandRegistry.insert(
		"editor.action.formatDocument".to_string(),
		CommandHandler::Native(CommandFormatDocument),
	);

	info!("[Bootstrap] {} native commands registered.", CommandRegistry.len());
	drop(CommandRegistry);

	// --- Tree View Provider Registration ---
	let mut TreeViewRegistry = ApplicationState.ActiveTreeViews.lock().unwrap();
	info!("[Bootstrap] Registering native tree view providers...");

	let ExplorerViewID = "workbench.view.explorer".to_string();
	let ExplorerProvider = Arc::new(FileExplorerViewProvider::new(AppHandle.clone()));

	TreeViewRegistry.insert(
		ExplorerViewID.clone(),
		TreeViewStateDTO {
			ViewIdentifier:ExplorerViewID,
			Provider:Some(ExplorerProvider),
			CanSelectMany:true,
			HasHandleDrag:false,
			HasHandleDrop:false,
			Message:None,
			Title:Some("Explorer".to_string()),
			Description:None,
		},
	);

	info!("[Bootstrap] {} native tree view providers registered.", TreeViewRegistry.len());
}
