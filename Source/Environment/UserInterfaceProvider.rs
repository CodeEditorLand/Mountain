// @module UiProvider (Environment)
// @description Implements the `UiProvider` trait for `MountainEnvironment`
// by delegating to the logic Handler in `Handler::ui`.

use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use Common::{
	Environment::Requires,
	error::CommonError,
	ui::{
		UiProvider,
		DTO::{
			InputBoxOptionsDTO,
			MessageSeverity,
			OpenDialogOptionsDTO,
			QuickPickItemDTO,
			QuickPickOptionsDTO,
			SaveDialogOptionsDTO,
		},
	},
};
use serde_json::Value;

use super::MountainEnvironment;
use crate::Handler::ui as UiHandler;

#[async_trait]
impl UiProvider for MountainEnvironment {
	// Handle showing a message by delegating to the `UiHandler`.
	async fn ShowMessage(
		&self,
		severity:MessageSeverity,
		message:String,
		options:Option<Value>,
	) -> Result<Option<String>, CommonError> {
		UiHandler::ShowMessageInteractiveLogic(&self.ApplicationHandle, severity, message, options).await
	}

	// Handle showing an open dialog by delegating to the `UiHandler`.
	async fn ShowOpenDialog(&self, options:Option<OpenDialogOptionsDTO>) -> Result<Option<Vec<PathBuf>>, CommonError> {
		UiHandler::ShowOpenDialogInteractiveLogic(&self.ApplicationHandle, options).await
	}

	// Handle showing a save dialog by delegating to the `UiHandler`.
	async fn ShowSaveDialog(&self, options:Option<SaveDialogOptionsDTO>) -> Result<Option<PathBuf>, CommonError> {
		UiHandler::ShowSaveDialogInteractiveLogic(&self.ApplicationHandle, options).await
	}

	// Handle showing a quick pick by delegating to the `UiHandler`.
	async fn ShowQuickPick(
		&self,
		items:Vec<QuickPickItemDTO>,
		options:Option<QuickPickOptionsDTO>,
	) -> Result<Option<Vec<String>>, CommonError> {
		UiHandler::ShowQuickPickInteractiveLogic(&self.ApplicationHandle, items, options).await
	}

	// Handle showing an input box by delegating to the `UiHandler`.
	async fn ShowInputBox(&self, options:Option<InputBoxOptionsDTO>) -> Result<Option<String>, CommonError> {
		UiHandler::ShowInputBoxInteractiveLogic(&self.ApplicationHandle, options).await
	}
}

impl Requires<Arc<dyn UiProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn UiProvider + Send + Sync> { Arc::new(self.clone()) }
}
