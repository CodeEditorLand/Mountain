#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use serde_json::{Value, json};
use tonic::Response;
use CommonLibrary::Secret::SecretProvider::SecretProvider;

use crate::{Environment::MountainEnvironment::MountainEnvironment, Vine::Generated::GenericResponse};

pub async fn Fn(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Key = Params.get("key").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let V = Params.get("value").and_then(|V| V.as_str()).unwrap_or("").to_string();

	match Env.StoreSecret(ExtensionId, Key, V).await {
		Ok(()) => super::super::FileSystem::OkResponse(RequestId, &json!({ "success": true })),

		Err(Error) => super::super::FileSystem::ErrResponse(RequestId, -32000, Error.to_string()),
	}
}
