//! Save Participants domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: participate_in_save.

use tonic::{Response, Status};
use ::Vine::Generated::{ParticipateInSaveRequest, ParticipateInSaveResponse};

use super::CocoonServiceImpl;
use crate::dev_log;

pub async fn Fn(
	Service:&CocoonServiceImpl,

	req:ParticipateInSaveRequest,
) -> Result<Response<ParticipateInSaveResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] Participating in save for: {:?}", req.uri);

	// Save participants are extension-registered onWillSaveTextDocument handlers.
	// Cocoon invokes this when an extension wants to participate in a save.
	// The extension has already computed its edits - they arrive via gRPC from
	// the Cocoon extension host. For now, pass through with no edits since
	// extension activation is not yet complete.
	dev_log!("cocoon", "[CocoonService] Save reason: {:?}, uri: {:?}", req.reason, req.uri);

	Ok(Response::new(ParticipateInSaveResponse { edits:Vec::new() }))
}
