use serde_json::{Value, json};
use tonic::Response;
use CommonLibrary::UserInterface::UserInterfaceProvider::UserInterfaceProvider;

use crate::{::Vine::Generated::GenericResponse, Environment::MountainEnvironment::MountainEnvironment};

pub async fn Fn(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	use CommonLibrary::UserInterface::DTO::InputBoxOptionsDTO::InputBoxOptionsDTO;

	let Opts = Params.get(0);

	let Options = InputBoxOptionsDTO {
		Prompt:Opts
			.and_then(|V| V.get("prompt"))
			.and_then(|P| P.as_str())
			.map(|S| S.to_string()),

		PlaceHolder:Opts
			.and_then(|V| V.get("placeHolder"))
			.and_then(|P| P.as_str())
			.map(|S| S.to_string()),

		IsPassword:Some(Opts.and_then(|V| V.get("password")).and_then(|B| B.as_bool()).unwrap_or(false)),

		Value:Opts
			.and_then(|V| V.get("value"))
			.and_then(|V| V.as_str())
			.map(|S| S.to_string()),

		Title:None,

		IgnoreFocusOut:None,
	};

	match Env.ShowInputBox(Some(Options)).await {
		Ok(Some(Text)) => super::super::FileSystem::OkResponse(RequestId, &json!(Text)),

		Ok(None) => super::super::FileSystem::OkResponse(RequestId, &Value::Null),

		Err(Error) => super::super::FileSystem::ErrResponse(RequestId, -32000, Error.to_string()),
	}
}
