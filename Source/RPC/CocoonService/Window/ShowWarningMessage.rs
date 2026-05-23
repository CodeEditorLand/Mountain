
//! Display a warning-severity message via the `UserInterfaceProvider`.

use tonic::{Response, Status};
use CommonLibrary::UserInterface::{
	DTO::MessageSeverity::MessageSeverity,
	UserInterfaceProvider::UserInterfaceProvider,
};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ShowMessageRequest, ShowMessageResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ShowMessageRequest,
) -> Result<Response<ShowMessageResponse>, Status> {
	dev_log!("cocoon", "warn: [CocoonService] show_warning_message: {}", Request.message);

	let _ = Service
		.environment
		.ShowMessage(MessageSeverity::Warning, Request.message, None)
		.await;

	Ok(Response::new(ShowMessageResponse { success:true }))
}
