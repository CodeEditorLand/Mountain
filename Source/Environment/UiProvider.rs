// File: Environment/UiProvider.rs
// Implements the `UiProvider` trait for the `MountainEnvironment`.
// This file connects abstract UI effects to the concrete logic
// in the application's UI handlers, which typically communicate with the
// frontend.

#![allow(non_snake_case, non_camel_case_types)]

use std::{path::PathBuf, sync::Arc};

use Common::{
	Environment::Requires,
	Errors::CommonError,
	UiEffect::{
		MessageSeverity,
		UiDto::{InputBoxOptions, OpenDialogOptions, QuickPickItem, QuickPickOptions, SaveDialogOptions},
		UiProvider,
	},
};
use async_trait::async_trait;
use serde_json::Value;

use crate::{Environment::MountainEnvironment, Handlers};

#[async_trait]
impl UiProvider for MountainEnvironment {
	/// Shows a message dialog to the user.
	async fn ShowMessage(
		&self,
		Severity:MessageSeverity,
		MessageText:String,
		OptionsJsonValueOption:Option<Value>,
	) -> Result<Option<String>, CommonError> {
		// The logic for this is complex (involving checking for simple dialogs vs. IPC
		// to Sky) and is fully contained within the handler. We delegate directly to
		// it.
		Handlers::Ui::HandleShowMessageInteractive(
			self.AppHandle.clone(),
			Severity,
			MessageText,
			OptionsJsonValueOption,
		)
		.await
	}

	/// Shows a native file open dialog.
	async fn ShowOpenDialog(&self, Options:Option<OpenDialogOptions>) -> Result<Option<Vec<PathBuf>>, CommonError> {
		Handlers::Ui::HandleShowOpenDialogInteractive(self.AppHandle.clone(), Options).await
	}

	/// Shows a native file save dialog.
	async fn ShowSaveDialog(&self, Options:Option<SaveDialogOptions>) -> Result<Option<PathBuf>, CommonError> {
		Handlers::Ui::HandleShowSaveDialogInteractive(self.AppHandle.clone(), Options).await
	}

	/// Shows a quick pick list to the user.
	async fn ShowQuickPick(
		&self,
		ItemList:Vec<QuickPickItem>,
		Options:Option<QuickPickOptions>,
	) -> Result<Option<Vec<String>>, CommonError> {
		Handlers::Ui::HandleShowQuickPickInteractive(self.AppHandle.clone(), ItemList, Options).await
	}

	/// Shows an input box to get text input from the user.
	async fn ShowInputBox(&self, Options:Option<InputBoxOptions>) -> Result<Option<String>, CommonError> {
		Handlers::Ui::HandleShowInputBoxInteractive(self.AppHandle.clone(), Options).await
	}
}

impl Requires<Arc<dyn UiProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn UiProvider + Send + Sync> { Arc::new(self.clone()) }
}
