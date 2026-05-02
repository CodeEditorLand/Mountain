#![allow(non_snake_case)]

//! Forward a task-termination request to Sky over
//! `sky://task/terminate`.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{Empty, TerminateTaskRequest},
	dev_log,
};

pub async fn Fn(Service:&CocoonServiceImpl, Request:TerminateTaskRequest) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] terminate_task: id={}", Request.task_id);

	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://task/terminate", json!({ "id": Request.task_id }));

	Ok(Response::new(Empty {}))
}
