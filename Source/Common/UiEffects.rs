// File: Common/UiEffects.rs
// Defines the UiProvider trait and associated effects for interacting with the
// user interface. This provides a standardized way to show dialogs,
// notifications, quick picks, and input boxes.

#![allow(non_snake_case, non_camel_case_types)]

use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use serde_json::Value;

use crate::{
	Effect::ActionEffect,
	Environment::{Environment, Requires},
	Errors::CommonError,
	Runtime::AppRuntimeTrait,
	UiDto::{
		InputBoxOptions,
		MessageOptions,
		MessageSeverity,
		OpenDialogOptions,
		QuickPickItem,
		QuickPickOptions,
		SaveDialogOptions,
	},
};

/// A trait for environments that can provide UI interaction capabilities.
#[async_trait]
pub trait UiProvider: Environment {
	/// Shows a message dialog to the user.
	async fn ShowMessage(
		&self,
		Severity:MessageSeverity,
		Message:String,
		Options:Option<Value>, // Using Value for flexibility with MessageOptions DTO
	) -> Result<Option<String>, CommonError>;

	/// Shows a native file open dialog.
	async fn ShowOpenDialog(&self, Options:Option<OpenDialogOptions>) -> Result<Option<Vec<PathBuf>>, CommonError>;

	/// Shows a native file save dialog.
	async fn ShowSaveDialog(&self, Options:Option<SaveDialogOptions>) -> Result<Option<PathBuf>, CommonError>;

	/// Shows a quick pick list to the user.
	async fn ShowQuickPick(
		&self,
		ItemList:Vec<QuickPickItem>,
		Options:Option<QuickPickOptions>,
	) -> Result<Option<Vec<String>>, CommonError>;

	/// Shows an input box to get text input from the user.
	async fn ShowInputBox(&self, Options:Option<InputBoxOptions>) -> Result<Option<String>, CommonError>;
}

/// Creates an effect to show a message dialog.
pub fn ShowMessage<RuntimeAccessType>(
	Severity:MessageSeverity,
	Message:String,
	OptionsValue:Value,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Option<String>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn UiProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let MessageClone = Message.clone();
		let OptionsClone = OptionsValue.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn UiProvider> = Environment.require();
			Provider.ShowMessage(Severity, MessageClone, Some(OptionsClone)).await
		})
	}))
}

/// Creates an effect to show a file open dialog.
pub fn ShowOpenDialog<RuntimeAccessType>(
	Options:Option<OpenDialogOptions>,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Option<Vec<PathBuf>>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn UiProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let OptionsClone = Options.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn UiProvider> = Environment.require();
			Provider.ShowOpenDialog(OptionsClone).await
		})
	}))
}

/// Creates an effect to show a file save dialog.
pub fn ShowSaveDialog<RuntimeAccessType>(
	Options:Option<SaveDialogOptions>,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Option<PathBuf>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn UiProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let OptionsClone = Options.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn UiProvider> = Environment.require();
			Provider.ShowSaveDialog(OptionsClone).await
		})
	}))
}

/// Creates an effect to show a quick pick list.
pub fn ShowQuickPick<RuntimeAccessType>(
	ItemList:Vec<QuickPickItem>,
	Options:Option<QuickPickOptions>,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Option<Vec<String>>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn UiProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let ItemListClone = ItemList.clone();
		let OptionsClone = Options.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn UiProvider> = Environment.require();
			Provider.ShowQuickPick(ItemListClone, OptionsClone).await
		})
	}))
}

/// Creates an effect to show an input box.
pub fn ShowInputBox<RuntimeAccessType>(
	Options:Option<InputBoxOptions>,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Option<String>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn UiProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let OptionsClone = Options.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn UiProvider> = Environment.require();
			Provider.ShowInputBox(OptionsClone).await
		})
	}))
}
