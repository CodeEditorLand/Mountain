#![allow(non_snake_case)]

//! Create a new output channel and notify Sky over `sky://output/create`.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{CreateOutputChannelRequest, CreateOutputChannelResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:CreateOutputChannelRequest,
) -> Result<Response<CreateOutputChannelResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] create_output_channel: '{}'", Request.name);

	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://output/create", json!({ "channel": Request.name }));

	Ok(Response::new(CreateOutputChannelResponse { channel_id:Request.name.clone() }))
}
