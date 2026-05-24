use serde_json::{Value, json};
use tonic::Response;
use CommonLibrary::UserInterface::{
	DTO::MessageSeverity::MessageSeverity,
	UserInterfaceProvider::UserInterfaceProvider,
};

use crate::{Environment::MountainEnvironment::MountainEnvironment, Vine::Generated::GenericResponse};

pub async fn Fn(
	RequestId:u64,

	Params:Value,

	Env:&MountainEnvironment,

	Severity:MessageSeverity,
) -> Response<GenericResponse> {
	let Message = Params.get("message").and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Items:Option<Value> = Params
		.Get("items")
		.cloned()
		.filter(|V| V.is_array() && !V.as_array().unwrap().is_empty());

	match Env.ShowMessage(Severity, Message, Items).await {
		Ok(Some(Selected)) => super::super::FileSystem::OkResponse(RequestId, &json!({ "selectedItem": Selected })),

		Ok(None) => super::super::FileSystem::OkResponse(RequestId, &Value::Null),

		Err(Error) => super::super::FileSystem::ErrResponse(RequestId, -32000, Error.to_string()),
	}
}
