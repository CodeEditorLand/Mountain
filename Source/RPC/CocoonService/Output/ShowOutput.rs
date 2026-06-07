//! Show an output channel in the workbench panel via
//! `sky://output/show`.

use serde_json::json;

use tauri::Emitter;

use tonic::{Response, Status};

use ::Vine::Generated::{Empty, ShowOutputRequest};

use crate::RPC::CocoonService::CocoonServiceImpl;

pub async fn Fn(Service:&CocoonServiceImpl, Request:ShowOutputRequest) -> Result<Response<Empty>, Status> {

	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://output/show", json!({ "channel": Request.channel_id }));

	Ok(Response::new(Empty {}))
}
