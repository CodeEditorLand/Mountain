//! All LSP feature method implementations.

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	LanguageFeature::DTO::{
		CompletionContextDTO::CompletionContextDTO,
		CompletionListDTO::CompletionListDTO,
		HoverResultDTO::HoverResultDTO,
		LocationDTO::LocationDTO,
		PositionDTO::PositionDTO,
		ProviderType::ProviderType,
		TextEditDTO::TextEditDTO,
	},
	IPC::IPCProvider::IPCProvider,
};
use crate::ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO;
use log::warn;
use serde_json::json;
use serde_json::Value;
use std::sync::Arc;
use url::Url;

// All feature methods delegate to generic invoke pattern

pub(super) async fn provide_code_actions(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	document_uri: Url,
	range_or_selection_dto: Value,
	context_dto: Value,
) -> Result<Option<Value>, CommonError> {
	let provider = super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::CodeAction).await?;
	match provider {
		Some(registration) => {
			let response = invoke_provider(environment, &registration, vec![
				json!(registration.Handle),
				json!({ "external": document_uri.to_string(), "$mid": 1 }),
				range_or_selection_dto,
				context_dto,
			]).await?;
			if response.is_null() {
				Ok(None)
			} else {
				Ok(Some(response))
			}
		}
		None => Ok(None),
	}
}

pub(super) async fn provide_code_lenses(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	document_uri: Url,
) -> Result<Option<Value>, CommonError> {
	let provider = super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::CodeLens).await?;
	match provider {
		Some(registration) => {
			let response = invoke_provider(environment, &registration, vec![
				json!(registration.Handle),
				json!({ "external": document_uri.to_string(), "$mid": 1 }),
			]).await?;
			if response.is_null() {
				Ok(None)
			} else {
				Ok(Some(response))
			}
		}
		None => Ok(None),
	}
}

pub(super) async fn provide_completions(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	document_uri: Url,
	position_dto: PositionDTO,
	context_dto: CompletionContextDTO,
	cancellation_token_value: Option<Value>,
) -> Result<Option<CompletionListDTO>, CommonError> {
	let provider = super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::Completion).await?;
	match provider {
		Some(registration) => {
			let response = invoke_provider(environment, &registration, vec![
				json!(registration.Handle),
				json!({ "external": document_uri.to_string(), "$mid": 1 }),
				json!(position_dto),
				json!(context_dto),
				cancellation_token_value.unwrap_or_else(|| json!(null)),
			]).await?;
			if response.is_null() {
				Ok(None)
			} else {
				serde_json::from_value(response).map_err(|error| CommonError::SerializationError { Description: format!("Failed to deserialize CompletionListDTO: {}", error) })
			}
		}
		None => Ok(None),
	}
}

pub(super) async fn provide_definition(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	document_uri: Url,
	position_dto: PositionDTO,
) -> Result<Option<Vec<LocationDTO>>, CommonError> {
	let provider = super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::Definition).await?;
	match provider {
		Some(registration) => {
			let response = invoke_provider(environment, &registration, vec![
				json!(registration.Handle),
				json!({ "external": document_uri.to_string(), "$mid": 1 }),
				json!(position_dto),
			]).await?;
			if response.is_null() {
				Ok(None)
			} else {
				serde_json::from_value(response).map_err(|error| CommonError::SerializationError { Description: format!("Failed to deserialize Vec<LocationDTO>: {}", error) })
			}
		}
		None => Ok(None),
	}
}

pub(super) async fn provide_document_formatting_edits(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	document_uri: Url,
	options_dto: Value,
) -> Result<Option<Vec<TextEditDTO>>, CommonError> {
	let provider = super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::DocumentFormatting).await?;
	match provider {
		Some(registration) => {
			let response = invoke_provider(environment, &registration, vec![
				json!(registration.Handle),
				json!({ "external": document_uri.to_string(), "$mid": 1 }),
				options_dto,
			]).await?;
			if response.is_null() {
				Ok(None)
			} else {
				serde_json::from_value(response).map_err(|error| CommonError::SerializationError { Description: format!("Failed to deserialize Vec<TextEditDTO>: {}", error) })
			}
		}
		None => Ok(None),
	}
}

pub(super) async fn provide_document_highlights(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	document_uri: Url,
	position_dto: PositionDTO,
) -> Result<Option<Value>, CommonError> {
	let provider = super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::DocumentHighlight).await?;
	match provider {
		Some(registration) => {
			let response = invoke_provider(environment, &registration, vec![
				json!(registration.Handle),
				json!({ "external": document_uri.to_string(), "$mid": 1 }),
				json!(position_dto),
			]).await?;
			if response.is_null() {
				Ok(None)
			} else {
				Ok(Some(response))
			}
		}
		None => Ok(None),
	}
}

pub(super) async fn provide_document_links(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	document_uri: Url,
) -> Result<Option<Value>, CommonError> {
	let provider = super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::DocumentLink).await?;
	match provider {
		Some(registration) => {
			let response = invoke_provider(environment, &registration, vec![
				json!(registration.Handle),
				json!({ "external": document_uri.to_string(), "$mid": 1 }),
			]).await?;
			if response.is_null() {
				Ok(None)
			} else {
				Ok(Some(response))
			}
		}
		None => Ok(None),
	}
}

pub(super) async fn provide_document_range_formatting_edits(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	document_uri: Url,
	range_dto: Value,
	options_dto: Value,
) -> Result<Option<Vec<TextEditDTO>>, CommonError> {
	let provider = super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::DocumentRangeFormatting).await?;
	match provider {
		Some(registration) => {
			let response = invoke_provider(environment, &registration, vec![
				json!(registration.Handle),
				json!({ "external": document_uri.to_string(), "$mid": 1 }),
				range_dto,
				options_dto,
			]).await?;
			if response.is_null() {
				Ok(None)
			} else {
				serde_json::from_value(response).map_err(|error| CommonError::SerializationError { Description: format!("Failed to deserialize Vec<TextEditDTO>: {}", error) })
			}
		}
		None => Ok(None),
	}
}

pub(super) async fn provide_hover(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	document_uri: Url,
	position_dto: PositionDTO,
) -> Result<Option<HoverResultDTO>, CommonError> {
	let provider = super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::Hover).await?;
	match provider {
		Some(registration) => {
			let response = invoke_provider(environment, &registration, vec![
				json!(registration.Handle),
				json!({ "external": document_uri.to_string(), "$mid": 1 }),
				json!(position_dto),
			]).await?;
			if response.is_null() {
				Ok(None)
			} else {
				serde_json::from_value(response).map_err(|error| CommonError::SerializationError { Description: format!("Failed to deserialize HoverResultDTO: {}", error) })
			}
		}
		None => Ok(None),
	}
}

pub(super) async fn provide_references(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	document_uri: Url,
	position_dto: PositionDTO,
	context_dto: Value,
) -> Result<Option<Vec<LocationDTO>>, CommonError> {
	let provider = super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::References).await?;
	match provider {
		Some(registration) => {
			let response = invoke_provider(environment, &registration, vec![
				json!(registration.Handle),
				json!({ "external": document_uri.to_string(), "$mid": 1 }),
				json!(position_dto),
				context_dto,
			]).await?;
			if response.is_null() {
				Ok(None)
			} else {
				serde_json::from_value(response).map_err(|error| CommonError::SerializationError { Description: format!("Failed to deserialize Vec<LocationDTO>: {}", error) })
			}
		}
		None => Ok(None),
	}
}

pub(super) async fn prepare_rename(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	document_uri: Url,
	position_dto: PositionDTO,
) -> Result<Option<Value>, CommonError> {
	let provider = super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::Rename).await?;
	match provider {
		Some(registration) => {
			let response = invoke_provider(environment, &registration, vec![
				json!(registration.Handle),
				json!({ "external": document_uri.to_string(), "$mid": 1 }),
				json!(position_dto),
			]).await?;
			if response.is_null() {
				Ok(None)
			} else {
				Ok(Some(response))
			}
		}
		None => Ok(None),
	}
}

async fn invoke_provider(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	registration: &ProviderRegistrationDTO,
	arguments: Vec<Value>,
) -> Result<Value, CommonError> {
	let rpc_method = format!("$provide{}", registration.ProviderType.to_string());
	let ipc_provider: Arc<dyn IPCProvider> = environment.Require();
	ipc_provider.SendRequestToSideCar(registration.SideCarIdentifier.clone(), rpc_method, json!(arguments), 5000).await
}
