// File: Mountain/Source/Environment/LanguageFeatureProvider.rs
// Role: Implements the `LanguageFeatureProviderRegistry` trait for the
// `MountainEnvironment`. Responsibilities:
//   - The central hub for all language intelligence features.
//   - Routes requests from the application to the appropriate extension
//     provider hosted in the `Cocoon` sidecar.
//   - Manages the registration and lifecycle of language providers.

//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//!
//! # LanguageFeatureProvider Implementation
//!
//! Implements the `LanguageFeatureProviderRegistry` trait for the
//! `MountainEnvironment`. This provider is the central hub for all language
//! intelligence features, routing requests from the application to the
//! appropriate extension provider hosted in the `Cocoon` sidecar.
//!
//! TODO (Mountain→Air Split): If Air provides advanced completion or indexing
//! services, consider adding a fallback provider chain: Air (cached/indexed)
//! → Cocoon (LSP) → Local (basic). Current implementation uses Cocoon only.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::IPCProvider::IPCProvider,
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
use log::{debug, info, warn};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use url::Url;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO;

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
		let Handle = self.ApplicationState.GetNextProviderHandle();

		info!(
			"[LangFeatureProvider] Registering {:?} provider from '{}' with new handle {}",
			ProviderType, SideCarIdentifier, Handle
		);

		let NewRegistration = ProviderRegistrationDTO {
			Handle,

			ProviderType,

			Selector:SelectorDTO,

			SideCarIdentifier,

			Options:OptionsDTO,

			ExtensionIdentifier:ExtensionIdentifierDTO,
		};

		self.ApplicationState
			.LanguageProviders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.insert(Handle, NewRegistration);

		Ok(Handle)
	}

	async fn UnregisterProvider(&self, Handle:u32) -> Result<(), CommonError> {
		info!("[LangFeatureProvider] Unregistering provider with handle {}", Handle);

		if self
			.ApplicationState
			.LanguageProviders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.remove(&Handle)
			.is_none()
		{
			warn!(
				"[LangFeatureProvider] Attempted to unregister a provider with handle {} that was not found.",
				Handle
			);
		}
		Ok(())
	}

	// --- Invocation Methods ---
	async fn ProvideHover(
		&self,

		DocumentURI:Url,

		PositionDTO:PositionDTO,
	) -> Result<Option<HoverResultDTO>, CommonError> {
		InvokeProvider(self, ProviderType::Hover, &DocumentURI, json!([PositionDTO])).await
	}

	async fn ProvideCompletions(
		&self,

		DocumentURI:Url,

		PositionDTO:PositionDTO,

		ContextDTO:CompletionContextDTO,

		CancellationTokenValue:Option<Value>,
	) -> Result<Option<CompletionListDTO>, CommonError> {
		InvokeProvider(
			self,
			ProviderType::Completion,
			&DocumentURI,
			json!([PositionDTO, ContextDTO, CancellationTokenValue]),
		)
		.await
	}

	async fn ProvideDefinition(
		&self,

		DocumentURI:Url,

		PositionDTO:PositionDTO,
	) -> Result<Option<Vec<LocationDTO>>, CommonError> {
		InvokeProvider(self, ProviderType::Definition, &DocumentURI, json!([PositionDTO])).await
	}

	async fn ProvideReferences(
		&self,

		DocumentURI:Url,

		PositionDTO:PositionDTO,

		ContextDTO:Value,
	) -> Result<Option<Vec<LocationDTO>>, CommonError> {
		InvokeProvider(self, ProviderType::References, &DocumentURI, json!([PositionDTO, ContextDTO])).await
	}

	async fn ProvideDocumentFormattingEdits(
		&self,

		DocumentURI:Url,

		OptionsDTO:Value,
	) -> Result<Option<Vec<TextEditDTO>>, CommonError> {
		InvokeProvider(self, ProviderType::DocumentFormatting, &DocumentURI, json!([OptionsDTO])).await
	}

	async fn ProvideDocumentRangeFormattingEdits(
		&self,

		DocumentURI:Url,

		RangeDTO:Value,

		OptionsDTO:Value,
	) -> Result<Option<Vec<TextEditDTO>>, CommonError> {
		InvokeProvider(
			self,
			ProviderType::DocumentRangeFormatting,
			&DocumentURI,
			json!([RangeDTO, OptionsDTO]),
		)
		.await
	}

	// --- Language Feature Provider Methods ---

	async fn ProvideCodeActions(
		&self,

		DocumentURI:Url,

		RangeOrSelectionDTO:Value,

		ContextDTO:Value,
	) -> Result<Option<Value>, CommonError> {
		InvokeProvider(
			self,
			ProviderType::CodeAction,
			&DocumentURI,
			json!([RangeOrSelectionDTO, ContextDTO]),
		)
		.await
	}

	async fn ProvideCodeLenses(&self, DocumentURI:Url) -> Result<Option<Value>, CommonError> {
		InvokeProvider(self, ProviderType::CodeLens, &DocumentURI, json!([Value::Null])).await
	}

	async fn ProvideDocumentHighlights(
		&self,

		DocumentURI:Url,

		PositionDTO:PositionDTO,
	) -> Result<Option<Value>, CommonError> {
		InvokeProvider(self, ProviderType::DocumentHighlight, &DocumentURI, json!([PositionDTO])).await
	}

	async fn ProvideDocumentLinks(&self, DocumentURI:Url) -> Result<Option<Value>, CommonError> {
		InvokeProvider(self, ProviderType::DocumentLink, &DocumentURI, json!([Value::Null])).await
	}

	async fn PrepareRename(&self, DocumentURI:Url, PositionDTO:PositionDTO) -> Result<Option<Value>, CommonError> {
		InvokeProvider(self, ProviderType::Rename, &DocumentURI, json!([PositionDTO])).await
	}
}

// --- Internal Helper for Invocation ---

/// Finds the best provider for a given feature and document.
fn FindBestProvider(
	Environment:&MountainEnvironment,

	ProviderType:ProviderType,

	DocumentURI:&Url,
) -> Result<Option<ProviderRegistrationDTO>, CommonError> {
	let Providers = Environment
		.ApplicationState
		.LanguageProviders
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

	let Document = Environment
		.ApplicationState
		.OpenDocuments
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
		.get(DocumentURI.as_str())
		.cloned();

	if let Some(doc) = Document {
		// This is a simplified selector matching logic. A real implementation would
		// score providers based on how well their DocumentSelector matches the
		// document.
		for Provider in Providers.values() {
			if Provider.ProviderType == ProviderType {
				if let Some(SelectorArray) = Provider.Selector.as_array() {
					for Selector in SelectorArray {
						if let Some(Lang) = Selector.get("language").and_then(Value::as_str) {
							if Lang == doc.LanguageIdentifier {
								debug!("Found provider with handle {} for document {}", Provider.Handle, DocumentURI);

								return Ok(Some(Provider.clone()));
							}
						}
						// TODO: Add scheme and pattern matching logic here.
						// Current implementation only matches language identifier.
						// Should also check:
						// - Selector["scheme"] (e.g., "file", "untitled", "custom")
						// - Selector["pattern"] (e.g., "**/*.ts", "src/**/*.rs")
						// - Selector["exclude"] (e.g., "node_modules/**")
						// Provider scoring should rank by specificity (pattern > language > all)
					}
				}
			}
		}
	}
	warn!("No provider found for {:?} on document {}", ProviderType, DocumentURI);

	Ok(None)
}

/// A generic helper to find the best provider, invoke it via RPC, and
/// deserialize the result.
async fn InvokeProvider<TResponse:DeserializeOwned>(
	Environment:&MountainEnvironment,

	ProviderType:ProviderType,

	DocumentURI:&Url,

	mut ProviderArguments:Value,
) -> Result<Option<TResponse>, CommonError> {
	if let Some(Provider) = FindBestProvider(Environment, ProviderType, DocumentURI)? {
		let RPCMethod = format!("$provide{}", Provider.ProviderType.to_string());

		let URIComponents = json!({ "external": DocumentURI.to_string(), "$mid": 1 });

		let ArgumentsVector = ProviderArguments.as_array_mut().ok_or_else(|| {
			CommonError::InvalidArgument {
				ArgumentName:"ProviderArguments".into(),

				Reason:"Expected provider arguments to be a JSON array.".into(),
			}
		})?;

		let mut FinalArgumentsVector = vec![json!(Provider.Handle), URIComponents];

		FinalArgumentsVector.append(ArgumentsVector);

		let FinalArguments = json!(FinalArgumentsVector);

		let IPCProvider:Arc<dyn IPCProvider> = Environment.Require();

		let Response = IPCProvider
			.SendRequestToSideCar(Provider.SideCarIdentifier, RPCMethod, FinalArguments, 5000)
			.await?;

		if Response.is_null() {
			return Ok(None);
		}
		serde_json::from_value(Response).map_err(|Error| {
			CommonError::SerializationError {
				Description:format!("Failed to deserialize response for {:?}: {}", ProviderType, Error),
			}
		})
	} else {
		Ok(None)
	}
}
