//! Display an input-box UI. Returns `cancelled:true` with empty value
//! when the user dismisses without confirming.
use tonic::{Response, Status};
use CommonLibrary::UserInterface::{
	DTO::InputBoxOptionsDTO::InputBoxOptionsDTO,
	UserInterfaceProvider::UserInterfaceProvider,
};
use ::Vine::Generated::{ShowInputBoxRequest, ShowInputBoxResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ShowInputBoxRequest,
) -> Result<Response<ShowInputBoxResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] show_input_box");

	let Options = Some(InputBoxOptionsDTO {
		Title:match Request.title.is_empty() {
			true => None,
			false => Some(Request.title),
		},
		PlaceHolder:match Request.placeholder.is_empty() {
			true => None,
			false => Some(Request.placeholder),
		},
		Value:match Request.value.is_empty() {
			true => None,
			false => Some(Request.value),
		},
		Prompt:match Request.prompt.is_empty() {
			true => None,
			false => Some(Request.prompt),
		},
		IsPassword:match Request.password {
			true => Some(true),
			false => None,
		},
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
