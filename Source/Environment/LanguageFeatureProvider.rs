//! # LanguageFeatureProvider Implementation
//!
//! Implements the `LanguageFeatureProviderRegistry` trait for the
//! `MountainEnvironment`. This provider is the central hub for all language
//! intelligence features, routing requests from the application to the
//! appropriate extension provider hosted in the `Cocoon` sidecar.

use std::sync::Arc;

use Common::{
	Error::CommonError::CommonError,
	IPC::IPCProvider::IPCProvider,
	LanguageFeature::{
		DTO::{HoverResultDTO::HoverResultDTO, PositionDTO::PositionDTO, ProviderType::ProviderType},
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
		SidecarIdentifier:String,
		ProviderType:ProviderType,
		SelectorDTO:Value,
		ExtensionIdentifierDTO:Value,
		OptionsDTO:Option<Value>,
	) -> Result<u32, CommonError> {
		let Handle = self.ApplicationState.GetNextProviderHandle();
		info!(
			"[LangFeatureProvider] Registering {:?} provider from '{}' with new handle {}",
			ProviderType, SidecarIdentifier, Handle
		);

		let NewRegistration = ProviderRegistrationDTO {
			Handle,
			ProviderType,
			Selector:SelectorDTO,
			SidecarIdentifier,
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
		ContextDTO:Value, // CompletionContextDTO
		CancellationTokenValue:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		// SuggestResultDTO
		InvokeProvider(
			self,
			ProviderType::Completion,
			&DocumentURI,
			json!([PositionDTO, ContextDTO, CancellationTokenValue]),
		)
		.await
	}

	// --- STUBS FOR ALL OTHER PROVIDER METHODS ---

	async fn PrepareCallHierarchy(
		&self,
		_DocumentURI:Url,
		_PositionDTO:PositionDTO,
		_CancellationTokenValue:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		warn!("[LangFeatureProvider] PrepareCallHierarchy is not implemented.");
		Ok(None)
	}

	async fn PrepareRename(
		&self,
		_DocumentURI:Url,
		_PositionDTO:PositionDTO,
		_CancellationTokenValue:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		warn!("[LangFeatureProvider] PrepareRename is not implemented.");
		Ok(None)
	}

	async fn PrepareTypeHierarchy(
		&self,
		_DocumentURI:Url,
		_PositionDTO:PositionDTO,
		_CancellationTokenValue:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		warn!("[LangFeatureProvider] PrepareTypeHierarchy is not implemented.");
		Ok(None)
	}

	async fn ProvideCallHierarchyIncomingCalls(
		&self,
		_ItemDTO:Value,
		_CancellationTokenValue:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		warn!("[LangFeatureProvider] ProvideCallHierarchyIncomingCalls is not implemented.");
		Ok(None)
	}

	async fn ProvideCallHierarchyOutgoingCalls(
		&self,
		_ItemDTO:Value,
		_CancellationTokenValue:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		warn!("[LangFeatureProvider] ProvideCallHierarchyOutgoingCalls is not implemented.");
		Ok(None)
	}

	async fn ProvideCodeActions(
		&self,
		_DocumentURI:Url,
		_RangeOrSelectionDTO:Value,
		_ContextDTO:Value,
		_CancellationTokenValue:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		warn!("[LangFeatureProvider] ProvideCodeActions is not implemented.");
		Ok(None)
	}

	async fn ProvideCodeLenses(
		&self,
		_DocumentURI:Url,
		_CancellationTokenValue:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		warn!("[LangFeatureProvider] ProvideCodeLenses is not implemented.");
		Ok(None)
	}

	async fn ProvideDocumentFormattingEdits(
		&self,
		_DocumentURI:Url,
		_OptionsDTO:Value,
		_CancellationTokenValue:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		warn!("[LangFeatureProvider] ProvideDocumentFormattingEdits is not implemented.");
		Ok(None)
	}

	async fn ProvideDocumentHighlights(
		&self,
		_DocumentURI:Url,
		_PositionDTO:PositionDTO,
		_CancellationTokenValue:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		warn!("[LangFeatureProvider] ProvideDocumentHighlights is not implemented.");
		Ok(None)
	}

	async fn ProvideDocumentLinks(
		&self,
		_DocumentURI:Url,
		_CancellationTokenValue:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		warn!("[LangFeatureProvider] ProvideDocumentLinks is not implemented.");
		Ok(None)
	}
}

// --- Internal Helper for Invocation ---

/// Finds the best provider for a given feature and document.
///
/// NOTE: This is a highly simplified stub. A real implementation needs a robust
/// system to parse the document selector (glob patterns, language ID, scheme)
/// and match it against the document's properties to score and select the best
/// provider.
fn FindBestProvider(
	Environment:&MountainEnvironment,
	ProviderType:ProviderType,
	DocumentURI:&Url,
) -> Option<ProviderRegistrationDTO> {
	let Providers = Environment.ApplicationState.LanguageProviders.lock().unwrap();
	// Extremely basic filtering logic for demonstration.
	for Provider in Providers.values() {
		if Provider.ProviderType == ProviderType {
			debug!("Found provider with handle {} for document {}", Provider.Handle, DocumentURI);
			return Some(Provider.clone());
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
		let RPCMethod = format!("${}", Provider.ProviderType.to_string());
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
			.SendRequestToSidecar(Provider.SidecarIdentifier, RPCMethod, FinalArguments, 5000)
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
