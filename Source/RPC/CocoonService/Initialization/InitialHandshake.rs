//! Cocoon → Mountain ready ping. No payload; Mountain replies with the
//! init-extension-host data via `InitExtensionHost::Fn`.

use tonic::{Response, Status};

use crate::{RPC::CocoonService::CocoonServiceImpl, Vine::Generated::Empty, dev_log};

pub async fn Fn(_Service:&CocoonServiceImpl, _Request:Empty) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Initial handshake received from Cocoon");

	Ok(Response::new(Empty {}))
}
