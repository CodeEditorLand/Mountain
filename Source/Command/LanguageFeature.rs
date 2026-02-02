// ============================================================================
// File: Mountain/Source/Command/LanguageFeature.rs
// ============================================================================
// # Language Feature Commands
//
//! Defines the specific Tauri command handlers for language feature requests
//! that originate from the `Sky` frontend UI (e.g., Monaco Editor).
//!
//! ## Key Features:
//! - LSP protocol wrapping and delegation
//! - Type-safe command parameter handling
//! - Hover information display
//! - Code completion suggestions
//! - Go-to-definition navigation
//! - References search
//!
//! ## VSCode Reference:
//! - vs/workbench/api/common/extHostLanguageFeatures.ts
//! - vs/workbench/services/languageFeatures/common/languageFeaturesService.ts
// ============================================================================

use std::sync::Arc;

use CommonLibrary::{
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

/// Validates language feature request parameters.
fn ValidateLanguageFeatureRequest(RequestType:&str, URI:&str, Position:&Value) -> Result<(), String> {
	if URI.is_empty() {
		return Err(format!("Empty URI for {} request", RequestType));
	}

	// Validate position format
	if let Some(Line) = Position.get("line") {
		if !Line.is_u64() {
			return Err(format!("Invalid line position for {} request", RequestType));
		}
	} else {
		return Err(format!("Missing line position for {} request", RequestType));
	}

	if let Some(Character) = Position.get("character") {
		if !Character.is_u64() {
			return Err(format!("Invalid character position for {} request", RequestType));
		}
	} else {
		return Err(format!("Missing character position for {} request", RequestType));
	}

	Ok(())
}

#[command]
pub async fn MountainProvideHover(
	ApplicationHandle:AppHandle<Wry>,

	URI:String,

	Position:Value,
) -> Result<Value, String> {
	log::debug!("[Language Feature] Providing hover for: {} at {:?}", URI, Position);

	ValidateLanguageFeatureRequest("hover", &URI, &Position)?;

	let DocumentURI = Url::parse(&URI).map_err(|Error| Error.to_string())?;

	let PositionDTO:PositionDTO =
		serde_json::from_value(Position.clone()).map_err(|Error| format!("Failed to parse position: {}", Error))?;

	InvokeProvider(ApplicationHandle, |Provider| {
		async move {
			let Result = Provider.ProvideHover(DocumentURI, PositionDTO).await?;

			Ok(serde_json::to_value(Result)?)
		}
	})
	.await
}

// #[command]
// pub async fn MountainProvideDocumentSymbols(ApplicationHandle:AppHandle<Wry>, URI:String) -> Result<Value, String> {
// 	log::debug!("[Language Feature] Providing document symbols for: {}", URI);

// 	if URI.is_empty() {
// 		return Err("Empty URI for document symbols request".to_string());
// 	}

// 	let DocumentURI = Url::parse(&URI).map_err(|Error| Error.to_string())?;

// 	InvokeProvider(ApplicationHandle, |Provider| {
// 		async move {
// 			let Result = Provider.ProvideDocumentSymbols(DocumentURI).await?;

// 			Ok(serde_json::to_value(Result)?)
// 		}
// 	})
// 	.await
// }

#[command]
pub async fn MountainProvideCodeActions(
	ApplicationHandle:AppHandle<Wry>,

	URI:String,

	Range:Value,

	Context:Value,
) -> Result<Value, String> {
	log::debug!("[Language Feature] Providing code actions for: {}", URI);

	if URI.is_empty() {
		return Err("Empty URI for code actions request".to_string());
	}

	let DocumentURI = Url::parse(&URI).map_err(|Error| Error.to_string())?;

	InvokeProvider(ApplicationHandle, |Provider| {
		async move {
			let Result = Provider.ProvideCodeActions(DocumentURI, Range, Context).await?;

			Ok(serde_json::to_value(Result)?)
		}
	})
	.await
}

#[command]
pub async fn MountainProvideDocumentHighlights(
	ApplicationHandle:AppHandle<Wry>,

	URI:String,

	Position:Value,
) -> Result<Value, String> {
	log::debug!("[Language Feature] Providing document highlights for: {}", URI);

	ValidateLanguageFeatureRequest("highlights", &URI, &Position)?;

	let DocumentURI = Url::parse(&URI).map_err(|Error| Error.to_string())?;

	let PositionDTO:PositionDTO =
		serde_json::from_value(Position.clone()).map_err(|Error| format!("Failed to parse position: {}", Error))?;

	InvokeProvider(ApplicationHandle, |Provider| {
		async move {
			let Result = Provider.ProvideDocumentHighlights(DocumentURI, PositionDTO).await?;

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
