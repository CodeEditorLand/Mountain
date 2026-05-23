#![allow(unused_variables, dead_code, unused_imports)]

use serde_json::{Value, json};
use tonic::Response;
use CommonLibrary::UserInterface::UserInterfaceProvider::UserInterfaceProvider;

use crate::{Environment::MountainEnvironment::MountainEnvironment, Vine::Generated::GenericResponse};

pub async fn Fn(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	use CommonLibrary::UserInterface::DTO::SaveDialogOptionsDTO::SaveDialogOptionsDTO;

	let Title = Params
		.get(0)
		.and_then(|V| V.get("title"))
		.and_then(|T| T.as_str())
		.map(|S| S.to_string());

	let Options = SaveDialogOptionsDTO {
		Base:CommonLibrary::UserInterface::DTO::DialogOptionsDTO::DialogOptionsDTO { Title, ..Default::default() },
		..SaveDialogOptionsDTO::default()
	};

	match Env.ShowSaveDialog(Some(Options)).await {
		Ok(Some(Path)) => super::super::FileSystem::OkResponse(RequestId, &json!(format!("file://{}", Path.display()))),

		Ok(None) => super::super::FileSystem::OkResponse(RequestId, &Value::Null),

		Err(Error) => super::super::FileSystem::ErrResponse(RequestId, -32000, Error.to_string()),
	}
}
