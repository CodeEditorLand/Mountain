#![allow(non_snake_case)]

//! Cancel an in-flight Mountain-originated operation by request id. Looks
//! up the cancellation token in `Service.ActiveOperations` and fires it.

use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{CancelOperationRequest, Empty},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:CancelOperationRequest) -> Result<Response<Empty>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] Cancel operation request: {}",
		Request.request_identifier_to_cancel
	);

	if let Some(Token) = Service.ActiveOperations.read().await.get(&Request.request_identifier_to_cancel) {
		dev_log!(
			"cocoon",
			"[CocoonService] Triggering cancellation token for operation {}",
			Request.request_identifier_to_cancel
		);

		Token.cancel();
	} else {
		dev_log!(
			"cocoon",
			"warn: [CocoonService] No active operation found for cancellation: {}",
			Request.request_identifier_to_cancel
		);
	}

	Ok(Response::new(Empty {}))
}
