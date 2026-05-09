#![allow(non_snake_case)]

//! Begin a progress notification. Mints a millisecond handle, emits
//! `sky://progress/start` so the workbench can render the bar.

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use tauri::Emitter;

use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ShowProgressRequest, ShowProgressResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ShowProgressRequest,
) -> Result<Response<ShowProgressResponse>, Status> {

	dev_log!("cocoon", "[CocoonService] show_progress: title={}", Request.title);

	let Handle = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|D| D.as_millis() as u32)
		.unwrap_or(0);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://progress/start",

		json!({
			"handle": Handle,
			"title": Request.title,
			"cancellable": Request.cancellable,
			"location": Request.location,
		}),
	);

	Ok(Response::new(ShowProgressResponse { handle:Handle }))
}
