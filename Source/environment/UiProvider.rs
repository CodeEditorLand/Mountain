use std::{path::PathBuf, sync::Arc};

use Common::{
	environment::Requires,
	error::CommonError,
	ui::{
		UiProvider,
		dto::{
			InputBoxOptionsDto,
			MessageSeverity,
			OpenDialogOptionsDto,
			QuickPickItemDto,
			QuickPickOptionsDto,
			SaveDialogOptionsDto,
		},
	},
};
use async_trait::async_trait;
use serde_json::Value;

/// @module UiProvider (Environment)
/// @description Implements the `UiProvider` trait for `MountainEnvironment`
/// by delegating to the logic handlers in `handlers::ui`.
use super::MountainEnvironment;
use crate::handlers::ui as UiHandler;

#[async_trait]
impl UiProvider for MountainEnvironment {
	/// Handles showing a message by delegating to the `UiHandler`.
	async fn ShowMessage(
		&self,
		Severity:MessageSeverity,
		Message:String,
		Options:Option<Value>,
	) -> Result<Option<String>, CommonError> {
		UiHandler::ShowMessageInteractiveLogic(&self.AppHandle, Severity, Message, Options).await
	}

	/// Handles showing an open dialog by delegating to the `UiHandler`.
	async fn ShowOpenDialog(&self, Options:Option<OpenDialogOptionsDto>) -> Result<Option<Vec<PathBuf>>, CommonError> {
		UiHandler::ShowOpenDialogInteractiveLogic(&self.AppHandle, Options).await
	}

	/// Handles showing a save dialog by delegating to the `UiHandler`.
	async fn ShowSaveDialog(&self, Options:Option<SaveDialogOptionsDto>) -> Result<Option<PathBuf>, CommonError> {
		UiHandler::ShowSaveDialogInteractiveLogic(&self.AppHandle, Options).await
	}

	/// Handles showing a quick pick by delegating to the `UiHandler`.
	async fn ShowQuickPick(
		&self,
		Items:Vec<QuickPickItemDto>,
		Options:Option<QuickPickOptionsDto>,
	) -> Result<Option<Vec<String>>, CommonError> {
		UiHandler::ShowQuickPickInteractiveLogic(&self.AppHandle, Items, Options).await
	}

	/// Handles showing an input box by delegating to the `UiHandler`.
	async fn ShowInputBox(&self, Options:Option<InputBoxOptionsDto>) -> Result<Option<String>, CommonError> {
		UiHandler::ShowInputBoxInteractiveLogic(&self.AppHandle, Options).await
	}
}

impl Requires<Arc<dyn UiProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn UiProvider + Send + Sync> { Arc::new(self.clone()) }
}
