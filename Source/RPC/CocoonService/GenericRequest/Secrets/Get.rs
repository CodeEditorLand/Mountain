use serde_json::{Value, json};
use tonic::Response;
use CommonLibrary::Secret::SecretProvider::SecretProvider;

use crate::{::Vine::Generated::GenericResponse, Environment::MountainEnvironment::MountainEnvironment};

pub async fn Fn(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	let ExtensionId = Params.get("extensionId").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Key = Params.get("key").and_then(|V| V.as_str()).unwrap_or("").to_string();

	match Env.GetSecret(ExtensionId, Key).await {
		Ok(Some(V)) => super::super::FileSystem::OkResponse(RequestId, &json!({ "value": V })),

		Ok(None) => super::super::FileSystem::OkResponse(RequestId, &Value::Null),

		Err(Error) => super::super::FileSystem::ErrResponse(RequestId, -32000, Error.to_string()),
	}
}
