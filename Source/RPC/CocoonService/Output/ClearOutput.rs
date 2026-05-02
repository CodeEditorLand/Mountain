#![allow(non_snake_case)]

//! Clear an output channel via `sky://output/clear`.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ClearOutputRequest, Empty},
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:ClearOutputRequest) -> Result<Response<Empty>, Status> {
	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://output/clear", json!({ "channel": Request.channel_id }));
	Ok(Response::new(Empty {}))
}
