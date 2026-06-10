//! Dispose an output channel via `sky://output/dispose`.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};
use ::Vine::Generated::{DisposeOutputRequest, Empty};

use crate::RPC::CocoonService::CocoonServiceImpl;

pub async fn Fn(Service:&CocoonServiceImpl, Request:DisposeOutputRequest) -> Result<Response<Empty>, Status> {
	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://output/dispose", json!({ "channel": Request.channel_id }));

	Ok(Response::new(Empty {}))
}
