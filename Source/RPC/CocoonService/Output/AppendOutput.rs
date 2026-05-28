//! Append text to an output channel via `sky://output/append`.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
};

use ::Vine::Generated::{AppendOutputRequest, Empty};

pub async fn Fn(Service:&CocoonServiceImpl, Request:AppendOutputRequest) -> Result<Response<Empty>, Status> {
	let _ = Service.environment.ApplicationHandle.emit(
		"sky://output/append",
		json!({ "channel": Request.channel_id, "text": Request.value }),
	);

	Ok(Response::new(Empty {}))
}
