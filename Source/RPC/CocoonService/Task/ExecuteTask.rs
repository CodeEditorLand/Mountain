//! Forward a task-execution request to Sky over the
//! `sky://task/execute` channel.
//!
//! Allocates a unique run-ID from the task execution registry, stores the
//! task definition JSON so `tasks:getTaskExecution` can return it later,
//! then emits the sky event and returns the real ID.
use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};
use ::Vine::Generated::{ExecuteTaskRequest, ExecuteTaskResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:ExecuteTaskRequest,
) -> Result<Response<ExecuteTaskResponse>, Status> {
	// Allocate a stable run-ID from the execution registry.
	let TaskId = Service.environment.ApplicationState.Feature.Tasks.NextId();

	dev_log!(
		"task",
		"[CocoonService] execute_task: id={} name={} source={}",
		TaskId,
		Request.name,
		Request.source
	);

	// Build the task definition to keep in the registry.
	let Definition = if let Some(ref Def) = Request.definition {
		json!({
			"id":     TaskId,
			"name":   Request.name,
			"source": Request.source,
			"type":   Def.r#type,
		})
	} else {
		json!({
			"id":     TaskId,
			"name":   Request.name,
			"source": Request.source,
		})
	};

	// Store so `tasks:getTaskExecution` can find it by ID.
	Service
		.environment
		.ApplicationState
		.Feature
		.Tasks
		.Insert(TaskId, Definition.clone());

	// Notify Sky to start the task in the workbench.
	let _ = Service.environment.ApplicationHandle.emit(
		"sky://task/execute",
		json!({
			"id":         TaskId,
			"name":       Request.name,
			"source":     Request.source,
			"definition": Definition,
		}),
	);

	Ok(Response::new(ExecuteTaskResponse { task_id:TaskId as u32, success:true }))
}
