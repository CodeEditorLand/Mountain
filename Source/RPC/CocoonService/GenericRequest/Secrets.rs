//! Generic-request secret-storage handlers for `process_mountain_request`.
//! Handles `getSecret`, `storeSecret`, `deleteSecret` using Cocoon's
//! `MountainGRPCClient` name conventions.
use CommonLibrary::Secret::SecretProvider::SecretProvider;
use serde_json::{Value, json};
use tonic::Response;
use ::Vine::Generated::GenericResponse;

use crate::Environment::MountainEnvironment::MountainEnvironment;
use super::FileSystem::{ErrResponse, OkResponse};

pub async fn HandleGetSecret(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Key = Params.get("key").and_then(|V| V.as_str()).unwrap_or("").to_string();

	match Env.GetSecret(ExtensionId, Key).await {
		Ok(Some(V)) => OkResponse(RequestId, &json!({ "value": V })),

		Ok(None) => OkResponse(RequestId, &Value::Null),

		Err(Error) => ErrResponse(RequestId, -32000, Error.to_string()),
	}
}

pub async fn HandleStoreSecret(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Key = Params.get("key").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let V = Params.get("value").and_then(|V| V.as_str()).unwrap_or("").to_string();

	match Env.StoreSecret(ExtensionId, Key, V).await {
		Ok(()) => OkResponse(RequestId, &json!({ "success": true })),

		Err(Error) => ErrResponse(RequestId, -32000, Error.to_string()),
	}
}

pub async fn HandleDeleteSecret(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Key = Params.get("key").and_then(|V| V.as_str()).unwrap_or("").to_string();

	match Env.DeleteSecret(ExtensionId, Key).await {
		Ok(()) => OkResponse(RequestId, &json!({ "success": true })),

		Err(Error) => ErrResponse(RequestId, -32000, Error.to_string()),
	}
}
