#![allow(unused_imports)]

//! # LanguageFeature (Command)
//!
//! RESPONSIBILITIES:
//! - Defines Tauri command handlers for language feature requests from Sky
//! frontend
//! - Bridges Monaco Editor language requests to
//! `LanguageFeatureProviderRegistry`
//! - Provides type-safe parameter handling and validation for LSP features
//! - Implements hover, code actions, document highlights, completions,
//! definition, references
//! - Uses generic `InvokeProvider` helper to reduce boilerplate
//!
//! ARCHITECTURAL ROLE:
//! - Command layer that exposes language features via Tauri IPC (`#[command]`)
//! - Delegates to Environment's
//! `LanguageFeatureProvider`
//! via DI with `Require()` trait
//! - Translates between frontend JSON parameters and Rust DTO types
//! - Error strings returned directly to frontend for display
//!
//! COMMAND REFERENCE (Tauri IPC):
//! - [`MountainProvideHover`]: Show hover information at cursor position
//! - [`MountainProvideCodeActions`]: Get quick fixes and refactorings for a
//!   code range
//! - [`MountainProvideDocumentHighlights`]: Find symbol occurrences in document
//! - [`MountainProvideCompletions`]: Get code completion suggestions with
//!   context
//! - [`MountainProvideDefinition`]: Jump to symbol definition location
//! - [`MountainProvideReferences`]: Find all references to a symbol
//!
//! ERROR HANDLING:
//! - Returns `Result<Value, String>` where errors sent directly to frontend
//! - Validates URI non-empty and position format (line/character numbers)
//! - JSON serialization errors converted to strings
//! - Provider errors (CommonError) converted to strings via `map_err(|Error|
//!   Error.to_string())`
//!
//! PERFORMANCE:
//! - Each command is async and non-blocking
//! - Provider lookup is O(1) via `Require()` from DI container
//! - URI parsing and DTO deserialization adds minimal overhead
//!
//! VS CODE REFERENCE:
//! - `vs/workbench/api/common/extHostLanguageFeatures.ts` - ext host language
//!   features API
//! - `vs/workbench/services/languageFeatures/common/languageFeaturesService.ts`
//!   - service layer
//! - `vs/workbench/contrib/hover/browser/hover.ts` - hover implementation
//! - `vs/workbench/contrib/completion/browser/completion.ts` - completion
//!   widget
//! - `vs/workbench/contrib/definition/browser/definition.ts` - go to definition
//! - `vs/workbench/contrib/references/browser/references.ts` - find references
//!
//! TODO:
//! - Implement more language features: document symbols, formatting, rename,
//!   signature help
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
//! MODULE STRUCTURE:
//! - [`validation.rs`](validation.rs) - request validation helper
//! - [`InvokeProvider.rs`](InvokeProvider.rs) - generic provider invoker
//! - Individual command modules for each language feature (containing impls
//!   only)

use serde_json::Value;
use tauri::{AppHandle, Wry, command};
use url::Url;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;
use crate::dev_log;

// Private submodules containing implementation (without #[command] attributes)
#[path = "LanguageFeature/validation.rs"]
mod validation;
#[path = "LanguageFeature/InvokeProvider.rs"]
mod InvokeProvider;
#[path = "LanguageFeature/hover.rs"]
mod hover;
#[path = "LanguageFeature/CodeActions.rs"]
mod CodeActions;
#[path = "LanguageFeature/highlights.rs"]
mod highlights;
#[path = "LanguageFeature/completions.rs"]
mod completions;
#[path = "LanguageFeature/definition.rs"]
mod definition;
#[path = "LanguageFeature/references.rs"]
mod references;

/// Provides hover information at cursor position
#[command]
pub async fn MountainProvideHover(
	application_handle:AppHandle<Wry>,
	uri:String,
	position:Value,
) -> Result<Value, String> {
	dev_log!("commands", "[Language Feature] Providing hover for: {} at {:?}", uri, position);
	hover::provide_hover_impl(application_handle, uri, position).await
}

/// Provides code actions (quick fixes and refactorings) for a code range
#[command]
pub async fn MountainProvideCodeActions(
	application_handle:AppHandle<Wry>,
	uri:String,
	position:Value,
	context:Value,
) -> Result<Value, String> {
	dev_log!("commands", "[Language Feature] Providing code actions for: {} at {:?}", uri, position);
	CodeActions::provide_CodeActions_impl(application_handle, uri, position, context).await
}

/// Finds symbol occurrences (document highlights) in a document
#[command]
pub async fn MountainProvideDocumentHighlights(
	application_handle:AppHandle<Wry>,
	uri:String,
	position:Value,
) -> Result<Value, String> {
	dev_log!("commands", 
		"[Language Feature] Providing document highlights for: {} at {:?}",
		uri, position
	);
	highlights::provide_document_highlights_impl(application_handle, uri, position).await
}

/// Provides code completion suggestions
#[command]
pub async fn MountainProvideCompletions(
	application_handle:AppHandle<Wry>,
	uri:String,
	position:Value,
	context:Value,
) -> Result<Value, String> {
	dev_log!("commands", "[Language Feature] Providing completions for: {} at {:?}", uri, position);
	completions::provide_completions_impl(application_handle, uri, position, context).await
}

/// Provides go-to-definition functionality
#[command]
pub async fn MountainProvideDefinition(
	application_handle:AppHandle<Wry>,
	uri:String,
	position:Value,
) -> Result<Value, String> {
	dev_log!("commands", "[Language Feature] Providing definition for: {} at {:?}", uri, position);
	definition::provide_definition_impl(application_handle, uri, position).await
}

/// Finds all references to a symbol
#[command]
pub async fn MountainProvideReferences(
	application_handle:AppHandle<Wry>,
	uri:String,
	position:Value,
	context:Value,
) -> Result<Value, String> {
	dev_log!("commands", "[Language Feature] Providing references for: {} at {:?}", uri, position);
	references::provide_references_impl(application_handle, uri, position, context).await
}
