//! Return an authentication session for the requested provider. Cocoon
//! auth providers register themselves via `RegisterAuthenticationProvider`
//! and live in `ApplicationState`; the full OAuth dance requires Mountain
//! to open a browser window, so for now we return an empty session.

use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	dev_log,
};

use ::Vine::Generated::{GetAuthenticationSessionRequest, GetAuthenticationSessionResponse};

pub async fn Fn(
	_Service:&CocoonServiceImpl,

	Request:GetAuthenticationSessionRequest,
) -> Result<Response<GetAuthenticationSessionResponse>, Status> {
	dev_log!(
		"cocoon",
		"[CocoonService] get_authentication_session: provider={}",
		Request.provider_id
	);

	Ok(Response::new(GetAuthenticationSessionResponse::default()))
}
