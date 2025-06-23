// File: Mountain/Source/Environment/LanguageFeatureProvider.rs
// Role: Implements the `LanguageFeatureProviderRegistry` trait for the
// `MountainEnvironment`. Responsibilities:
//   - The central hub for all language intelligence features.
//   - Routes requests from the application to the appropriate extension
//     provider hosted in the `Cocoon` sidecar.
//   - Manages the registration and lifecycle of language providers.

//! # LanguageFeatureProvider Implementation
//!
//! Implements the `LanguageFeatureProviderRegistry` trait for the
//! `MountainEnvironment`. This provider is the central hub for all language
//! intelligence features, routing requests from the application to the
//! appropriate extension provider hosted in the `Cocoon` sidecar.

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

	// --- STUBS FOR OTHER PROVIDER METHODS ---
	async fn ProvideCodeActions(
		&self,

		_DocumentURI:Url,

		_RangeOrSelectionDTO:Value,

		_ContextDTO:Value,
	) -> Result<Option<Value>, CommonError> {
		warn!("[LangFeatureProvider] ProvideCodeActions is not implemented.");

		Ok(None)
	}

	async fn ProvideCodeLenses(&self, _DocumentURI:Url) -> Result<Option<Value>, CommonError> {
		warn!("[LangFeatureProvider] ProvideCodeLenses is not implemented.");

		Ok(None)
	}

	async fn ProvideDocumentHighlights(
		&self,

		_DocumentURI:Url,

		_PositionDTO:PositionDTO,
	) -> Result<Option<Value>, CommonError> {
		warn!("[LangFeatureProvider] ProvideDocumentHighlights is not implemented.");

		Ok(None)
	}

	async fn ProvideDocumentLinks(&self, _DocumentURI:Url) -> Result<Option<Value>, CommonError> {
		warn!("[LangFeatureProvider] ProvideDocumentLinks is not implemented.");

		Ok(None)
	}

	async fn PrepareRename(&self, _DocumentURI:Url, _PositionDTO:PositionDTO) -> Result<Option<Value>, CommonError> {
		warn!("[LangFeatureProvider] PrepareRename is not implemented.");

		Ok(None)
	}
}

// --- Internal Helper for Invocation ---

/// Finds the best provider for a given feature and document.
fn FindBestProvider(
	Environment:&MountainEnvironment,

	ProviderType:ProviderType,

	DocumentURI:&Url,
) -> Option<ProviderRegistrationDTO> {
	let Providers = Environment.ApplicationState.LanguageProviders.lock().unwrap();

	let Document = Environment
		.ApplicationState
		.OpenDocuments
		.lock()
		.unwrap()
		.get(DocumentURI.as_str())
		.cloned();

	if let Some(doc) = Document {
		// This is a simplified selector matching logic. A real implementation would
		// score providers based on how well their DocumentSelector matches the
		// document (scheme, pattern, language).
		for Provider in Providers.values() {
			if Provider.ProviderType == ProviderType {
				if let Some(selector_array) = Provider.Selector.as_array() {
					for selector in selector_array {
						if let Some(lang) = selector.get("language").and_then(Value::as_str) {
							if lang == doc.LanguageIdentifier {
								debug!("Found provider with handle {} for document {}", Provider.Handle, DocumentURI);

								return Some(Provider.clone());
							}
						}

						// TODO: Add scheme and pattern matching logic here.
					}
				}
			}
		}
	}

	warn!("No provider found for {:?} on document {}", ProviderType, DocumentURI);

	None
}

/// A generic helper to find the best provider, invoke it via RPC, and
/// deserialize the result.
async fn InvokeProvider<TResponse:DeserializeOwned>(
	Environment:&MountainEnvironment,

	ProviderType:ProviderType,

	DocumentURI:&Url,

	mut ProviderArguments:Value,
) -> Result<Option<TResponse>, CommonError> {
	if let Some(Provider) = FindBestProvider(Environment, ProviderType, DocumentURI) {
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

		serde_json::from_value(Response).map_err(|e| {
			CommonError::SerializationError {
				Description:format!("Failed to deserialize response for {:?}: {}", ProviderType, e),
			}
		})
	} else {
		Ok(None)
	}
}
