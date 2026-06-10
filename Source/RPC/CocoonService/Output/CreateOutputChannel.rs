//! Create a new output channel and notify Sky over `sky://output/create`.

use serde_json::json;

use tauri::Emitter;

use tonic::{Response, Status};

use ::Vine::Generated::{CreateOutputChannelRequest, CreateOutputChannelResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:CreateOutputChannelRequest,
) -> Result<Response<CreateOutputChannelResponse>, Status> {

	dev_log!("cocoon", "[CocoonService] create_output_channel: '{}'", Request.name);

	// Sky's InstallEditorAndOutput.ts destructures { id, name }.
	// The old `{ channel }` key made both fields undefined, keying the
	// output channel on the string "undefined" in Sky's map.
	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://output/create", json!({ "id": Request.name, "name": Request.name }));

	Ok(Response::new(CreateOutputChannelResponse { channel_id:Request.name.clone() }))
}
