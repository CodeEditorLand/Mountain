#![allow(non_snake_case)]
//! Output channel domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: create_output_channel, append_output, clear_output,
//! show_output, dispose_output.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use super::CocoonServiceImpl;
use crate::{
	Vine::Generated::{
		AppendOutputRequest,
		ClearOutputRequest,
		CreateOutputChannelRequest,
		CreateOutputChannelResponse,
		DisposeOutputRequest,
		Empty,
		ShowOutputRequest,
	},
	dev_log,
};

pub async fn CreateOutputChannel(
	Service:&CocoonServiceImpl,
	req:CreateOutputChannelRequest,
) -> Result<Response<CreateOutputChannelResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] create_output_channel: '{}'", req.name);

	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://output/create", json!({ "channel": req.name }));

	Ok(Response::new(CreateOutputChannelResponse { channel_id:req.name.clone() }))
}

pub async fn AppendOutput(Service:&CocoonServiceImpl, req:AppendOutputRequest) -> Result<Response<Empty>, Status> {
	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://output/append", json!({ "channel": req.channel_id, "text": req.value }));
	Ok(Response::new(Empty {}))
}

pub async fn ClearOutput(Service:&CocoonServiceImpl, req:ClearOutputRequest) -> Result<Response<Empty>, Status> {
	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://output/clear", json!({ "channel": req.channel_id }));
	Ok(Response::new(Empty {}))
}

pub async fn ShowOutput(Service:&CocoonServiceImpl, req:ShowOutputRequest) -> Result<Response<Empty>, Status> {
	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://output/show", json!({ "channel": req.channel_id }));
	Ok(Response::new(Empty {}))
}

pub async fn DisposeOutput(Service:&CocoonServiceImpl, req:DisposeOutputRequest) -> Result<Response<Empty>, Status> {
	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://output/dispose", json!({ "channel": req.channel_id }));
	Ok(Response::new(Empty {}))
}
