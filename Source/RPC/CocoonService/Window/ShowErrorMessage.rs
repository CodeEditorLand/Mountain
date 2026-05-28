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

	let _ = Service
		.environment
		.ShowMessage(MessageSeverity::Error, Request.message, None)
		.await;

	Ok(Response::new(ShowMessageResponse { success:true }))
}
