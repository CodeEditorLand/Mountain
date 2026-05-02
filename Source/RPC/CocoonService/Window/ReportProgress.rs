#![allow(non_snake_case)]

//! Update a progress notification with a new message + increment.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, ReportProgressRequest},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:ReportProgressRequest) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] report_progress: handle={}", Request.handle);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://progress/update",
		json!({
			"handle": Request.handle,
			"message": Request.message,
			"increment": Request.increment,
		}),
	);

	Ok(Response::new(Empty {}))
}
