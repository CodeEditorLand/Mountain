//! Display an error-severity message via the `UserInterfaceProvider`.
use tonic::{Response, Status};
use CommonLibrary::UserInterface::{
	DTO::MessageSeverity::MessageSeverity,
	UserInterfaceProvider::UserInterfaceProvider,
};
use ::Vine::Generated::{ShowMessageRequest, ShowMessageResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ShowMessageRequest,
) -> Result<Response<ShowMessageResponse>, Status> {
	dev_log!("cocoon", "error: [CocoonService] show_error_message: {}", Request.message);

	let Items:Option<serde_json::Value> = match Request.items.is_empty() {
		true => None,

		false => Some(serde_json::json!(Request.items)),
	};

	let _ = Service
		.environment
		.ShowMessage(MessageSeverity::Error, Request.message, Items)
		.await;

	Ok(Response::new(ShowMessageResponse { success:true }))
}
