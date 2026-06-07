//! # LanguageFeatureProvider (Environment)
//!
//! Provides language feature intelligence through extension-hosted LSP
//! providers. Manages provider registration, lookup, and invocation for hover,
//! completion, definition, references, formatting, code actions, and more.
//!
//! ## Implementation Strategy
//!
//! The trait implementation is split across multiple helper modules:
//! - `Registration`: RegisterProvider, UnregisterProvider
//! - `ProviderLookup`: GetMatchingProvider (private helper)
//! - `FeatureMethods`: All LSP feature methods (Hover, Completion, etc.)
//!
//! The single `impl LanguageFeatureProviderRegistry for MountainEnvironment`
//! block in this file delegates to those helper functions. This satisfies
//! Rust's orphan rules while keeping code organized and atomic.

use CommonLibrary::{
	Error::CommonError::CommonError,
	LanguageFeature::{
		DTO::{
			CompletionContextDTO::CompletionContextDTO,
			CompletionListDTO::CompletionListDTO,
			HoverResultDTO::HoverResultDTO,
			LocationDTO::LocationDTO,
			PositionDTO::PositionDTO,
			ProviderType::ProviderType,
			TextEditDTO::TextEditDTO,
		},
		LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	},
};
use async_trait::async_trait;
use serde_json::Value;
use url::Url;

use crate::Environment::MountainEnvironment::MountainEnvironment;

// Private helper modules (not re-exported)
mod Registration;

mod ProviderLookup;

mod FeatureMethods;

#[async_trait]
impl LanguageFeatureProviderRegistry for MountainEnvironment {
	async fn RegisterProvider(
		&self,

		SideCarIdentifier:String,

		ProviderType:ProviderType,

		SelectorDTO:Value,

		ExtensionIdentifierDTO:Value,

		OptionsDTO:Option<Value>,
	) -> Result<u32, CommonError> {
		Registration::register_provider(
			self,
			SideCarIdentifier,
			ProviderType,
			SelectorDTO,
			ExtensionIdentifierDTO,
			OptionsDTO,
		)
		.await
	}

	async fn UnregisterProvider(&self, Handle:u32) -> Result<(), CommonError> {
		Registration::unregister_provider(self, Handle).await
	}

	async fn ProvideCodeActions(
		&self,

		DocumentURI:Url,

		RangeOrSelectionDTO:Value,

		ContextDTO:Value,
	) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_code_actions(self, DocumentURI, RangeOrSelectionDTO, ContextDTO).await
	}

	async fn ProvideCodeLenses(&self, DocumentURI:Url) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_code_lenses(self, DocumentURI).await
	}

	async fn ProvideCompletions(
		&self,

		DocumentURI:Url,

		PositionDTO:PositionDTO,

		ContextDTO:CompletionContextDTO,

		CancellationTokenValue:Option<Value>,
	) -> Result<Option<CompletionListDTO>, CommonError> {
		FeatureMethods::provide_completions(self, DocumentURI, PositionDTO, ContextDTO, CancellationTokenValue).await
	}

	async fn ProvideDefinition(
		&self,

		DocumentURI:Url,

		PositionDTO:PositionDTO,
	) -> Result<Option<Vec<LocationDTO>>, CommonError> {
		FeatureMethods::provide_definition(self, DocumentURI, PositionDTO).await
	}

	async fn ProvideDocumentFormattingEdits(
		&self,

		DocumentURI:Url,

		OptionsDTO:Value,
	) -> Result<Option<Vec<TextEditDTO>>, CommonError> {
		FeatureMethods::provide_document_formatting_edits(self, DocumentURI, OptionsDTO).await
	}

	async fn ProvideDocumentHighlights(
		&self,

		DocumentURI:Url,

		PositionDTO:PositionDTO,
	) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_document_highlights(self, DocumentURI, PositionDTO).await
	}

	async fn ProvideDocumentLinks(&self, DocumentURI:Url) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_document_links(self, DocumentURI).await
	}

	async fn ProvideDocumentRangeFormattingEdits(
		&self,

		DocumentURI:Url,

		RangeDTO:Value,

		OptionsDTO:Value,
	) -> Result<Option<Vec<TextEditDTO>>, CommonError> {
		FeatureMethods::provide_document_range_formatting_edits(self, DocumentURI, RangeDTO, OptionsDTO).await
	}

	async fn ProvideHover(
		&self,

		DocumentURI:Url,

		PositionDTO:PositionDTO,
	) -> Result<Option<HoverResultDTO>, CommonError> {
		FeatureMethods::provide_hover(self, DocumentURI, PositionDTO).await
	}

	async fn ProvideReferences(
		&self,

		DocumentURI:Url,

		PositionDTO:PositionDTO,

		ContextDTO:Value,
	) -> Result<Option<Vec<LocationDTO>>, CommonError> {
		FeatureMethods::provide_references(self, DocumentURI, PositionDTO, ContextDTO).await
	}

	async fn PrepareRename(&self, DocumentURI:Url, PositionDTO:PositionDTO) -> Result<Option<Value>, CommonError> {
		FeatureMethods::prepare_rename(self, DocumentURI, PositionDTO).await
	}

	async fn ProvideRenameEdits(
		&self,

		DocumentURI:Url,

		PositionDTO:PositionDTO,

		NewName:String,
	) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_rename_edits(self, DocumentURI, PositionDTO, NewName).await
	}

	async fn ProvideDocumentSymbols(&self, DocumentURI:Url) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_document_symbols(self, DocumentURI).await
	}

	async fn ProvideWorkspaceSymbols(&self, Query:String) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_workspace_symbols(self, Query).await
	}

	async fn ProvideSignatureHelp(
		&self,

		DocumentURI:Url,

		PositionDTO:PositionDTO,

		ContextDTO:Value,
	) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_signature_help(self, DocumentURI, PositionDTO, ContextDTO).await
	}

	async fn ProvideFoldingRanges(&self, DocumentURI:Url) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_folding_ranges(self, DocumentURI).await
	}

	async fn ProvideSelectionRanges(
		&self,

		DocumentURI:Url,

		Positions:Vec<PositionDTO>,
	) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_selection_ranges(self, DocumentURI, Positions).await
	}

	async fn ProvideSemanticTokensFull(&self, DocumentURI:Url) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_semantic_tokens_full(self, DocumentURI).await
	}

	async fn ProvideInlayHints(&self, DocumentURI:Url, RangeDTO:Value) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_inlay_hints(self, DocumentURI, RangeDTO).await
	}

	async fn PrepareCallHierarchy(
		&self,

		DocumentURI:Url,

		PositionDTO:PositionDTO,
	) -> Result<Option<Value>, CommonError> {
		FeatureMethods::prepare_call_hierarchy(self, DocumentURI, PositionDTO).await
	}

	async fn PrepareTypeHierarchy(
		&self,

		DocumentURI:Url,

		PositionDTO:PositionDTO,
	) -> Result<Option<Value>, CommonError> {
		FeatureMethods::prepare_type_hierarchy(self, DocumentURI, PositionDTO).await
	}

	async fn ProvideTypeHierarchySupertypes(&self, ItemDTO:Value) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_type_hierarchy_supertypes(self, ItemDTO).await
	}

	async fn ProvideTypeHierarchySubtypes(&self, ItemDTO:Value) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_type_hierarchy_subtypes(self, ItemDTO).await
	}

	async fn ProvideCallHierarchyIncomingCalls(&self, ItemDTO:Value) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_call_hierarchy_incoming_calls(self, ItemDTO).await
	}

	async fn ProvideCallHierarchyOutgoingCalls(&self, ItemDTO:Value) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_call_hierarchy_outgoing_calls(self, ItemDTO).await
	}

	async fn ProvideLinkedEditingRanges(
		&self,

		DocumentURI:Url,

		PositionDTO:PositionDTO,
	) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_linked_editing_ranges(self, DocumentURI, PositionDTO).await
	}

	async fn ProvideFileDecoration(&self, ResourceURI:Url) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_file_decoration(self, ResourceURI).await
	}

	async fn ProvideInlineCompletionItems(
		&self,

		DocumentURI:Url,

		PositionDTO_:PositionDTO,

		ContextDTO:Value,
	) -> Result<Option<Value>, CommonError> {
		FeatureMethods::provide_inline_completion_items(self, DocumentURI, PositionDTO_, ContextDTO).await
	}

	async fn ProvideOnTypeFormattingEdits(
		&self,

		DocumentURI:Url,

		PositionDTO:PositionDTO,

		Character:String,

		OptionsDTO:Value,
	) -> Result<Option<Vec<TextEditDTO>>, CommonError> {
		FeatureMethods::provide_on_type_formatting_edits(self, DocumentURI, PositionDTO, Character, OptionsDTO).await
	}
}
