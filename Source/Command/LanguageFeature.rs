//! # LanguageFeature (Command)
//!
//! RESPONSIBILITIES:
//! - Defines Tauri command handlers for language feature requests from Sky frontend
//! - Bridges Monaco Editor language requests to [`LanguageFeatureProviderRegistry`]
//! - Provides type-safe parameter handling and validation for LSP features
//! - Implements hover, code actions, document highlights, completions, definition, references
//! - Uses generic `InvokeProvider` helper to reduce boilerplate
//!
//! ARCHITECTURAL ROLE:
//! - Command layer that exposes language features via Tauri IPC (`#[command]`)
//! - Delegates to Environment's [`LanguageFeatureProvider`](crate::Environment::LanguageFeatureProvider)
//!   via DI with `Require()` trait
//! - Translates between frontend JSON parameters and Rust DTO types
//! - Error strings returned directly to frontend for display
//!
//! COMMAND REFERENCE (Tauri IPC):
//! - [`MountainProvideHover`](crate::Command::LanguageFeature::MountainProvideHover):
//!   Show hover information at cursor position
//! - [`MountainProvideCodeActions`](crate::Command::LanguageFeature::MountainProvideCodeActions):
//!   Get quick fixes and refactorings for a code range
//! - [`MountainProvideDocumentHighlights`](crate::Command::LanguageFeature::MountainProvideDocumentHighlights):
//!   Find symbol occurrences in document
//! - [`MountainProvideCompletions`](crate::Command::LanguageFeature::MountainProvideCompletions):
//!   Get code completion suggestions with context
//! - [`MountainProvideDefinition`](crate::Command::LanguageFeature::MountainProvideDefinition):
//!   Jump to symbol definition location
//! - [`MountainProvideReferences`](crate::Command::LanguageFeature::MountainProvideReferences):
//!   Find all references to a symbol
//!
//! ERROR HANDLING:
//! - Returns `Result<Value, String>` where errors sent directly to frontend
//! - Validates URI non-empty and position format (line/character numbers)
//! - JSON serialization errors converted to strings
//! - Provider errors (CommonError) converted to strings via `map_err(|Error| Error.to_string())`
//!
//! PERFORMANCE:
//! - Each command is async and non-blocking
//! - Provider lookup is O(1) via `Require()` from DI container
//! - URI parsing and DTO deserialization adds minimal overhead
//!
//! VS CODE REFERENCE:
//! - `vs/workbench/api/common/extHostLanguageFeatures.ts` - ext host language features API
//! - `vs/workbench/services/languageFeatures/common/languageFeaturesService.ts` - service layer
//! - `vs/workbench/contrib/hover/browser/hover.ts` - hover implementation
//! - `vs/workbench/contrib/completion/browser/completion.ts` - completion widget
//! - `vs/workbench/contrib/definition/browser/definition.ts` - go to definition
//! - `vs/workbench/contrib/references/browser/references.ts` - find references
//!
//! TODO:
//! - Implement more language features: document symbols, formatting, rename, signature help
//! - Add cancellation token support for long-running operations
//! - Implement request deduplication for identical concurrent requests
//! - Add request caching for repeated symbol lookups
//! - Support workspace symbol search
//! - Add semantic tokens for syntax highlighting
//! - Implement code lens provider
//! - Add inlay hints support
//! - Support linked editing range
//! - Add call hierarchy and type hierarchy
//! - Implement document color and color presentation
//! - Add folding range provider
//! - Support selection range provider
//!
//! MODULE CONTENTS:
//! - Helper function: `InvokeProvider<F, T>` - generic provider invoker
//! - Validation function: `ValidateLanguageFeatureRequest`
//! - Tauri command functions (all `#[command] pub async fn`):
//!   - Hover: `MountainProvideHover`
//!   - Code Actions: `MountainProvideCodeActions`
//!   - Highlights: `MountainProvideDocumentHighlights`
//!   - Completions: `MountainProvideCompletions`
//!   - Definition: `MountainProvideDefinition`
//!   - References: `MountainProvideReferences`
//! - (Commented out: `MountainProvideDocumentSymbols`)

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
// pub async fn MountainProvideDocumentSymbols(ApplicationHandle:AppHandle<Wry>,
// URI:String) -> Result<Value, String> { 	log::debug!("[Language Feature]
// Providing document symbols for: {}", URI);

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
