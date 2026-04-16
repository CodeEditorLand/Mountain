#![allow(non_snake_case)]
//! Task domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: register_task_provider, execute_task, terminate_task.

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use super::CocoonServiceImpl;
use crate::ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO;
use crate::dev_log;
use crate::Vine::Generated::{
	Empty, ExecuteTaskRequest, ExecuteTaskResponse,
	RegisterTaskProviderRequest, TerminateTaskRequest,
};
use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;

pub async fn RegisterTaskProvider(
	Service:&CocoonServiceImpl,
	req:RegisterTaskProviderRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering Task Provider: type={}", req.r#type);

	// Task providers don't have handles in proto — use a hash of the type string
	let Handle = req.r#type.as_bytes().iter().fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32));
	let dto = ProviderRegistrationDTO {
		Handle,
		ProviderType:ProviderType::Task,
		Selector:json!([{ "language": "*" }]),
		SideCarIdentifier:"cocoon-main".to_string(),
		ExtensionIdentifier:json!(req.extension_id),
		Options:None,
	};
	Service
		.environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.RegisterProvider(Handle, dto);

	Ok(Response::new(Empty {}))
}

pub async fn ExecuteTask(
	Service:&CocoonServiceImpl,
	req:ExecuteTaskRequest,
) -> Result<Response<ExecuteTaskResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] execute_task: name={} source={}", req.name, req.source);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://task/execute",
		json!({ "name": req.name, "source": req.source }),
	);

	Ok(Response::new(ExecuteTaskResponse { task_id:0, success:true }))
}

pub async fn TerminateTask(
	Service:&CocoonServiceImpl,
	req:TerminateTaskRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] terminate_task: id={}", req.task_id);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://task/terminate",
		json!({ "id": req.task_id }),
	);

	Ok(Response::new(Empty {}))
}
