//! # Language Feature Commands
//!
//! Defines the specific Tauri command handlers for language feature requests
//! that originate from the `Sky` frontend UI (e.g., Monaco Editor).

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{
	Environment::Requires::Requires,
	LanguageFeature::{
		DTO::{CompletionContextDTO::CompletionContextDTO, PositionDTO::PositionDTO},
		LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	},
};
use serde_json::Value;
use tauri::{AppHandle, Manager, command};
use url::Url;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime as MountainRunTime;

#[command]
pub async fn MountainProvideHover(ApplicationHandle:AppHandle, URI:String, Position:Value) -> Result<Value, String> {
	let RunTime = ApplicationHandle.state::<Arc<MountainRunTime>>().inner().clone();
	let Provider:Arc<dyn LanguageFeatureProviderRegistry> = RunTime.Environment.Require();
	let DocumentURI = Url::parse(&URI).map_err(|e| e.to_string())?;
	let PositionDTO:PositionDTO = serde_json::from_value(Position).map_err(|e| e.to_string())?;

	let Result = Provider
		.ProvideHover(DocumentURI, PositionDTO)
		.await
		.map_err(|e| e.to_string())?;
	Ok(serde_json::to_value(Result).unwrap_or(Value::Null))
}

#[command]
pub async fn MountainProvideCompletions(
	ApplicationHandle:AppHandle,
	URI:String,
	Position:Value,
	Context:Value,
) -> Result<Value, String> {
	let RunTime = ApplicationHandle.state::<Arc<MountainRunTime>>().inner().clone();
	let Provider:Arc<dyn LanguageFeatureProviderRegistry> = RunTime.Environment.Require();
	let DocumentURI = Url::parse(&URI).map_err(|e| e.to_string())?;
	let PositionDTO:PositionDTO = serde_json::from_value(Position).map_err(|e| e.to_string())?;
	let ContextDTO:CompletionContextDTO = serde_json::from_value(Context).map_err(|e| e.to_string())?;

	let Result = Provider
		.ProvideCompletions(DocumentURI, PositionDTO, ContextDTO, None)
		.await
		.map_err(|e| e.to_string())?;
	Ok(serde_json::to_value(Result).unwrap_or(Value::Null))
}

#[command]
pub async fn MountainProvideDefinition(
	ApplicationHandle:AppHandle,
	URI:String,
	Position:Value,
) -> Result<Value, String> {
	let RunTime = ApplicationHandle.state::<Arc<MountainRunTime>>().inner().clone();
	let Provider:Arc<dyn LanguageFeatureProviderRegistry> = RunTime.Environment.Require();
	let DocumentURI = Url::parse(&URI).map_err(|e| e.to_string())?;
	let PositionDTO:PositionDTO = serde_json::from_value(Position).map_err(|e| e.to_string())?;

	let Result = Provider
		.ProvideDefinition(DocumentURI, PositionDTO)
		.await
		.map_err(|e| e.to_string())?;
	Ok(serde_json::to_value(Result).unwrap_or(Value::Null))
}

#[command]
pub async fn MountainProvideReferences(
	ApplicationHandle:AppHandle,
	URI:String,
	Position:Value,
	Context:Value,
) -> Result<Value, String> {
	let RunTime = ApplicationHandle.state::<Arc<MountainRunTime>>().inner().clone();
	let Provider:Arc<dyn LanguageFeatureProviderRegistry> = RunTime.Environment.Require();
	let DocumentURI = Url::parse(&URI).map_err(|e| e.to_string())?;
	let PositionDTO:PositionDTO = serde_json::from_value(Position).map_err(|e| e.to_string())?;

	let Result = Provider
		.ProvideReferences(DocumentURI, PositionDTO, Context)
		.await
		.map_err(|e| e.to_string())?;
	Ok(serde_json::to_value(Result).unwrap_or(Value::Null))
}
