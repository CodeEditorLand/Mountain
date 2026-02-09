//! # LanguageFeatureProvider (Environment)
//!
//! Provides language feature intelligence through extension-hosted LSP
//! providers. Manages provider registration, lookup, and invocation for hover,
//! completion, definition, references, formatting, code actions, and more.
//!
//! ## Implementation Strategy
//!
//! The trait implementation is split across multiple helper modules:
//! - [`Registration`]: RegisterProvider, UnregisterProvider
//! - [`ProviderLookup`]: GetMatchingProvider (private helper)
//! - [`FeatureMethods`]: All LSP feature methods (Hover, Completion, etc.)
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

use crate::{
	ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO,
	Environment::MountainEnvironment::MountainEnvironment,
};

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
}
