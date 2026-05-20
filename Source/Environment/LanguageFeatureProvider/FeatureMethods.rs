//! All LSP feature method implementations.

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::IPCProvider::IPCProvider,
	LanguageFeature::DTO::{
		CompletionContextDTO::CompletionContextDTO,
		CompletionListDTO::CompletionListDTO,
		HoverResultDTO::HoverResultDTO,
		LocationDTO::LocationDTO,
		PositionDTO::PositionDTO,
		ProviderType::ProviderType,
		TextEditDTO::TextEditDTO,
	},
};
use serde_json::{Value, json};
use url::Url;

use crate::ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO;

// All feature methods delegate to generic invoke pattern

pub(super) async fn provide_code_actions(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	range_or_selection_dto:Value,

	context_dto:Value,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::CodeAction).await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					range_or_selection_dto,
					context_dto,
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_code_lenses(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::CodeLens).await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_completions(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,

	context_dto:CompletionContextDTO,

	cancellation_token_value:Option<Value>,
) -> Result<Option<CompletionListDTO>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::Completion).await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					json!(position_dto),
					json!(context_dto),
					cancellation_token_value.unwrap_or_else(|| json!(null)),
				],
			)
			.await?;

			if response.is_null() {
				Ok(None)
			} else {
				serde_json::from_value(response).map_err(|error| {
					CommonError::SerializationError {
						Description:format!("Failed to deserialize CompletionListDTO: {}", error),
					}
				})
			}
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_definition(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,
) -> Result<Option<Vec<LocationDTO>>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::Definition).await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					json!(position_dto),
				],
			)
			.await?;

			if response.is_null() {
				Ok(None)
			} else {
				serde_json::from_value(response).map_err(|error| {
					CommonError::SerializationError {
						Description:format!("Failed to deserialize Vec<LocationDTO>: {}", error),
					}
				})
			}
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_document_formatting_edits(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	options_dto:Value,
) -> Result<Option<Vec<TextEditDTO>>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::DocumentFormatting)
			.await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					options_dto,
				],
			)
			.await?;

			if response.is_null() {
				Ok(None)
			} else {
				serde_json::from_value(response).map_err(|error| {
					CommonError::SerializationError {
						Description:format!("Failed to deserialize Vec<TextEditDTO>: {}", error),
					}
				})
			}
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_document_highlights(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::DocumentHighlight)
			.await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					json!(position_dto),
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_document_links(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::DocumentLink).await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_document_range_formatting_edits(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	range_dto:Value,

	options_dto:Value,
) -> Result<Option<Vec<TextEditDTO>>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::DocumentRangeFormatting)
			.await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					range_dto,
					options_dto,
				],
			)
			.await?;

			if response.is_null() {
				Ok(None)
			} else {
				serde_json::from_value(response).map_err(|error| {
					CommonError::SerializationError {
						Description:format!("Failed to deserialize Vec<TextEditDTO>: {}", error),
					}
				})
			}
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_hover(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,
) -> Result<Option<HoverResultDTO>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::Hover).await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					json!(position_dto),
				],
			)
			.await?;

			if response.is_null() {
				Ok(None)
			} else {
				serde_json::from_value(response).map_err(|error| {
					CommonError::SerializationError {
						Description:format!("Failed to deserialize HoverResultDTO: {}", error),
					}
				})
			}
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_references(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,

	context_dto:Value,
) -> Result<Option<Vec<LocationDTO>>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::References).await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					json!(position_dto),
					context_dto,
				],
			)
			.await?;

			if response.is_null() {
				Ok(None)
			} else {
				serde_json::from_value(response).map_err(|error| {
					CommonError::SerializationError {
						Description:format!("Failed to deserialize Vec<LocationDTO>: {}", error),
					}
				})
			}
		},

		None => Ok(None),
	}
}

pub(super) async fn prepare_rename(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::Rename).await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					json!(position_dto),
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_rename_edits(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,

	new_name:String,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::Rename).await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					json!(position_dto),
					json!(new_name),
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_document_symbols(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::DocumentSymbol).await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_workspace_symbols(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	query:String,
) -> Result<Option<Value>, CommonError> {
	// Workspace symbols don't have a specific document URI - use a dummy lookup.
	// The provider is registered globally, so we pick the first WorkspaceSymbol
	// provider.
	let MatchingRegistration = {
		let providers = environment
			.ApplicationState
			.Extension
			.ProviderRegistration
			.LanguageProviders
			.lock()
			.map_err(crate::Environment::Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

		providers
			.values()
			.find(|p| p.ProviderType == ProviderType::WorkspaceSymbol)
			.cloned()
	};

	match MatchingRegistration {
		Some(registration) => {
			let response =
				invoke_provider(environment, &registration, vec![json!(registration.Handle), json!(query)]).await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_signature_help(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,

	context_dto:Value,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::SignatureHelp).await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					json!(position_dto),
					context_dto,
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_folding_ranges(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::FoldingRange).await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_selection_ranges(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	positions:Vec<PositionDTO>,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::SelectionRange).await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					json!(positions),
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_semantic_tokens_full(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::SemanticTokens).await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_inlay_hints(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	range_dto:Value,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::InlayHint).await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					range_dto,
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_type_hierarchy_supertypes(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	item_dto:Value,
) -> Result<Option<Value>, CommonError> {
	// Type hierarchy uses the item's URI to find the provider
	let uri_str = item_dto.get("uri").and_then(|u| u.as_str()).unwrap_or("");

	let document_uri = Url::parse(uri_str).unwrap_or_else(|_| Url::parse("file:///unknown").unwrap());

	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::TypeHierarchy).await?;

	match provider {
		Some(registration) => {
			let response =
				invoke_provider(environment, &registration, vec![json!(registration.Handle), item_dto]).await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_type_hierarchy_subtypes(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	item_dto:Value,
) -> Result<Option<Value>, CommonError> {
	let uri_str = item_dto.get("uri").and_then(|u| u.as_str()).unwrap_or("");

	let document_uri = Url::parse(uri_str).unwrap_or_else(|_| Url::parse("file:///unknown").unwrap());

	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::TypeHierarchy).await?;

	match provider {
		Some(registration) => {
			let response =
				invoke_provider(environment, &registration, vec![json!(registration.Handle), item_dto]).await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

/// Prepare call hierarchy - establish the root `CallHierarchyItem` at the
/// given document position. Extensions implement `prepareCallHierarchy(doc,
/// pos, token)`. Without this step the incoming/outgoing calls views are always
/// empty.
pub(super) async fn prepare_call_hierarchy(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::CallHierarchy).await?;

	match provider {
		Some(registration) => {
			let uri_json = json!({ "external": document_uri.to_string(), "$mid": 1 });
			let pos_json = json!({ "Line": position_dto.LineNumber, "Character": position_dto.Column });
			let response = invoke_provider_method(
				environment,
				&registration,
				"$prepareCallHierarchyItems",
				vec![json!(registration.Handle), uri_json, pos_json],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

/// Prepare type hierarchy - establish the root `TypeHierarchyItem`.
pub(super) async fn prepare_type_hierarchy(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::TypeHierarchy).await?;

	match provider {
		Some(registration) => {
			let uri_json = json!({ "external": document_uri.to_string(), "$mid": 1 });
			let pos_json = json!({ "Line": position_dto.LineNumber, "Character": position_dto.Column });
			let response = invoke_provider_method(
				environment,
				&registration,
				"$prepareTypeHierarchyItems",
				vec![json!(registration.Handle), uri_json, pos_json],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_call_hierarchy_incoming_calls(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	item_dto:Value,
) -> Result<Option<Value>, CommonError> {
	let uri_str = item_dto.get("uri").and_then(|u| u.as_str()).unwrap_or("");

	let document_uri = Url::parse(uri_str).unwrap_or_else(|_| Url::parse("file:///unknown").unwrap());

	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::CallHierarchy).await?;

	match provider {
		Some(registration) => {
			let response =
				invoke_provider(environment, &registration, vec![json!(registration.Handle), item_dto]).await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_call_hierarchy_outgoing_calls(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	item_dto:Value,
) -> Result<Option<Value>, CommonError> {
	let uri_str = item_dto.get("uri").and_then(|u| u.as_str()).unwrap_or("");

	let document_uri = Url::parse(uri_str).unwrap_or_else(|_| Url::parse("file:///unknown").unwrap());

	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::CallHierarchy).await?;

	match provider {
		Some(registration) => {
			let response =
				invoke_provider(environment, &registration, vec![json!(registration.Handle), item_dto]).await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_linked_editing_ranges(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,
) -> Result<Option<Value>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::LinkedEditingRange)
			.await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					json!(position_dto),
				],
			)
			.await?;

			if response.is_null() { Ok(None) } else { Ok(Some(response)) }
		},

		None => Ok(None),
	}
}

pub(super) async fn provide_on_type_formatting_edits(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	document_uri:Url,

	position_dto:PositionDTO,

	character:String,

	options_dto:Value,
) -> Result<Option<Vec<TextEditDTO>>, CommonError> {
	let provider =
		super::ProviderLookup::get_matching_provider(environment, &document_uri, ProviderType::OnTypeFormatting)
			.await?;

	match provider {
		Some(registration) => {
			let response = invoke_provider(
				environment,
				&registration,
				vec![
					json!(registration.Handle),
					json!({ "external": document_uri.to_string(), "$mid": 1 }),
					json!(position_dto),
					json!(character),
					options_dto,
				],
			)
			.await?;

			if response.is_null() {
				Ok(None)
			} else {
				serde_json::from_value(response).map_err(|error| {
					CommonError::SerializationError {
						Description:format!("Failed to deserialize Vec<TextEditDTO>: {}", error),
					}
				})
			}
		},

		None => Ok(None),
	}
}

async fn invoke_provider(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	registration:&ProviderRegistrationDTO,

	arguments:Vec<Value>,
) -> Result<Value, CommonError> {
	let rpc_method = format!("$provide{}", registration.ProviderType.to_string());

	let ipc_provider:Arc<dyn IPCProvider> = environment.Require();

	ipc_provider
		.SendRequestToSideCar(registration.SideCarIdentifier.clone(), rpc_method, json!(arguments), 5000)
		.await
}

/// Like `invoke_provider` but uses an explicit method name instead of
/// the `$provide{ProviderType}` convention. Used for prepare steps
/// (`$prepareCallHierarchyItems`, `$prepareTypeHierarchyItems`) where
/// the method prefix differs from the provider type string.
async fn invoke_provider_method(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	registration:&ProviderRegistrationDTO,

	method:&str,

	arguments:Vec<Value>,
) -> Result<Value, CommonError> {
	let ipc_provider:Arc<dyn IPCProvider> = environment.Require();

	ipc_provider
		.SendRequestToSideCar(
			registration.SideCarIdentifier.clone(),
			method.to_string(),
			json!(arguments),
			5000,
		)
		.await
}
