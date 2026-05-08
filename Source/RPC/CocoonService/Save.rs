#![allow(non_snake_case)]
//! Save Participants domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: participate_in_save.

use tonic::{Response, Status};

use super::CocoonServiceImpl;
use crate::{
	Vine::Generated::{ParticipateInSaveRequest, ParticipateInSaveResponse},
	dev_log,
};

pub async fn ParticipateInSave(
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
