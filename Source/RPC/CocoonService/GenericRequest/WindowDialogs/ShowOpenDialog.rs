#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use serde_json::{Value, json};
use tonic::Response;
use CommonLibrary::UserInterface::UserInterfaceProvider::UserInterfaceProvider;

use crate::{Environment::MountainEnvironment::MountainEnvironment, Vine::Generated::GenericResponse};

pub async fn Fn(RequestId:u64, Params:Value, Env:&MountainEnvironment) -> Response<GenericResponse> {
	use CommonLibrary::UserInterface::DTO::OpenDialogOptionsDTO::OpenDialogOptionsDTO;

	let Title = Params
		.get(0)
		.and_then(|V| V.get("title"))
		.and_then(|T| T.as_str())
		.map(|S| S.to_string());

	let Options = OpenDialogOptionsDTO {
		Base:CommonLibrary::UserInterface::DTO::DialogOptionsDTO::DialogOptionsDTO { Title, ..Default::default() },
		..OpenDialogOptionsDTO::default()
	};

	match Env.ShowOpenDialog(Some(Options)).await {
		Ok(Some(Paths)) => {
			let Uris:Vec<String> = Paths.iter().map(|P| format!("file://{}", P.display())).collect();

			super::super::FileSystem::OkResponse(RequestId, &json!(Uris))
		},

		Ok(None) => super::super::FileSystem::OkResponse(RequestId, &json!(serde_json::Value::Array(vec![]))),

		Err(Error) => super::super::FileSystem::ErrResponse(RequestId, -32000, Error.to_string()),
	}
}
