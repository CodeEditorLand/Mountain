//! # Language Feature Commands
//!
//! Defines the specific Tauri command handlers for language feature requests
//! that originate from the `Sky` frontend UI (e.g., Monaco Editor).

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	LanguageFeature::{
		DTO::{CompletionContextDTO::CompletionContextDTO, PositionDTO::PositionDTO},
		LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	},
};
use serde_json::Value;
use tauri::{AppHandle, Manager, Wry, command};
use url::Url;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime as MountainRunTime;

/// A generic helper to reduce boilerplate in language feature command handlers.
async fn InvokeProvider<F, T>(ApplicationHandle:AppHandle<Wry>, Handler:F) -> Result<Value, String>
where
	F: FnOnce(Arc<dyn LanguageFeatureProviderRegistry>) -> T,
	T: std::future::Future<Output = Result<Value, CommonError>>, {
	let RunTime = ApplicationHandle.state::<Arc<MountainRunTime>>().inner().clone();

	let Provider:Arc<dyn LanguageFeatureProviderRegistry> = RunTime.Environment.Require();

	let Result = Handler(Provider).await.map_err(|Error| Error.to_string())?;

	serde_json::to_value(Result).map_err(|Error| Error.to_string())
}

#[command]
pub async fn MountainProvideHover(
	ApplicationHandle:AppHandle<Wry>,

	URI:String,

	Position:Value,
) -> Result<Value, String> {
	let DocumentURI = Url::parse(&URI).map_err(|Error| Error.to_string())?;

	let PositionDTO:PositionDTO = serde_json::from_value(Position).map_err(|Error| Error.to_string())?;

	InvokeProvider(ApplicationHandle, |Provider| {
		async move {
			let Result = Provider.ProvideHover(DocumentURI, PositionDTO).await?;

			Ok(serde_json::to_value(Result)?)
		}
	})
	.await
}

#[command]
pub async fn MountainProvideCompletions(
	ApplicationHandle:AppHandle<Wry>,

	URI:String,

	Position:Value,

	Context:Value,
) -> Result<Value, String> {
	let DocumentURI = Url::parse(&URI).map_err(|Error| Error.to_string())?;

	let PositionDTO:PositionDTO = serde_json::from_value(Position).map_err(|Error| Error.to_string())?;

	let ContextDTO:CompletionContextDTO = serde_json::from_value(Context).map_err(|Error| Error.to_string())?;

	InvokeProvider(ApplicationHandle, |Provider| {
		async move {
			let Result = Provider.ProvideCompletions(DocumentURI, PositionDTO, ContextDTO, None).await?;

			Ok(serde_json::to_value(Result)?)
		}
	})
	.await
}

#[command]
pub async fn MountainProvideDefinition(
	ApplicationHandle:AppHandle<Wry>,

	URI:String,

	Position:Value,
) -> Result<Value, String> {
	let DocumentURI = Url::parse(&URI).map_err(|Error| Error.to_string())?;

	let PositionDTO:PositionDTO = serde_json::from_value(Position).map_err(|Error| Error.to_string())?;

	InvokeProvider(ApplicationHandle, |Provider| {
		async move {
			let Result = Provider.ProvideDefinition(DocumentURI, PositionDTO).await?;

			Ok(serde_json::to_value(Result)?)
		}
	})
	.await
}

#[command]
pub async fn MountainProvideReferences(
	ApplicationHandle:AppHandle<Wry>,

	URI:String,

	Position:Value,

	Context:Value,
) -> Result<Value, String> {
	let DocumentURI = Url::parse(&URI).map_err(|Error| Error.to_string())?;

	let PositionDTO:PositionDTO = serde_json::from_value(Position).map_err(|Error| Error.to_string())?;

	InvokeProvider(ApplicationHandle, |Provider| {
		async move {
			let Result = Provider.ProvideReferences(DocumentURI, PositionDTO, Context).await?;

			Ok(serde_json::to_value(Result)?)
		}
	})
	.await
}
