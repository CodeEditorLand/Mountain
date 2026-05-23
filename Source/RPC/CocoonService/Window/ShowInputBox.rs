//! Display an input-box UI. Returns `cancelled:true` with empty value
//! when the user dismisses without confirming.

use tonic::{Response, Status};
use CommonLibrary::UserInterface::{
	DTO::InputBoxOptionsDTO::InputBoxOptionsDTO,
	UserInterfaceProvider::UserInterfaceProvider,
};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ShowInputBoxRequest, ShowInputBoxResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ShowInputBoxRequest,
) -> Result<Response<ShowInputBoxResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] show_input_box");

	let Options = Some(InputBoxOptionsDTO {
		Title:if Request.title.is_empty() { None } else { Some(Request.title) },
		PlaceHolder:if Request.placeholder.is_empty() { None } else { Some(Request.placeholder) },
		Value:if Request.value.is_empty() { None } else { Some(Request.value) },
		Prompt:if Request.prompt.is_empty() { None } else { Some(Request.prompt) },
		IsPassword:if Request.password { Some(true) } else { None },
		IgnoreFocusOut:None,
	});

	match Service.environment.ShowInputBox(Options).await {
		Ok(Some(Value)) => Ok(Response::new(ShowInputBoxResponse { value:Value, cancelled:false })),

		Ok(None) => Ok(Response::new(ShowInputBoxResponse { value:String::new(), cancelled:true })),

		Err(Error) => {
			dev_log!("cocoon", "warn: [CocoonService] show_input_box failed: {}", Error);

			Ok(Response::new(ShowInputBoxResponse { value:String::new(), cancelled:true }))
		},
	}
}
