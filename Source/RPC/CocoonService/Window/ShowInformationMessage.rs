//! Display an info-severity message via the `UserInterfaceProvider`.
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
	dev_log!("cocoon", "[CocoonService] show_information_message: {}", Request.message);

	let Items:Option<serde_json::Value> = if Request.items.is_empty() {
		None
	} else {
		Some(serde_json::json!(Request.items))
	};

	let _ = Service
		.environment
		.ShowMessage(MessageSeverity::Info, Request.message, Items)
		.await;

	Ok(Response::new(ShowMessageResponse { success:true }))
}
