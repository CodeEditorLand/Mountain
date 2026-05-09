#![allow(non_snake_case)]

//! Forward a task-execution request to Sky over the
//! `sky://task/execute` channel.

use serde_json::json;

use tauri::Emitter;

use tonic::{Response, Status};

use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	Vine::Generated::{ExecuteTaskRequest, ExecuteTaskResponse},
	dev_log,
};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ExecuteTaskRequest,
) -> Result<Response<ExecuteTaskResponse>, Status> {

	dev_log!(
		"cocoon",

		"[CocoonService] execute_task: name={} source={}",

		Request.name,

		Request.source
	);

	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://task/execute", json!({ "name": Request.name, "source": Request.source }));

	Ok(Response::new(ExecuteTaskResponse { task_id:0, success:true }))
}
