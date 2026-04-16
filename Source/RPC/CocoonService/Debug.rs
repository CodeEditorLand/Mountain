#![allow(non_snake_case)]
//! Debug domain handlers for CocoonService.
//!
//! Typed gRPC RPCs: register_debug_adapter, start_debugging,
//! stop_debugging.

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};

use super::CocoonServiceImpl;
use crate::ApplicationState::DTO::ProviderRegistrationDTO::ProviderRegistrationDTO;
use crate::dev_log;
use crate::Vine::Generated::{
	Empty, RegisterDebugAdapterRequest, StartDebuggingRequest,
	StartDebuggingResponse, StopDebuggingRequest,
};
use CommonLibrary::LanguageFeature::DTO::ProviderType::ProviderType;

pub async fn RegisterDebugAdapter(
	Service:&CocoonServiceImpl,
	req:RegisterDebugAdapterRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] Registering debug adapter: {}", req.debug_type);

	let Handle = req.debug_type.as_bytes().iter().fold(0u32, |Acc, B| Acc.wrapping_mul(31).wrapping_add(*B as u32));
	let dto = ProviderRegistrationDTO {
		Handle,
		ProviderType:ProviderType::DebugAdapter,
		Selector:json!([{ "debugType": req.debug_type }]),
		SideCarIdentifier:"cocoon-main".to_string(),
		ExtensionIdentifier:json!(req.extension_id),
		Options:Some(json!({ "debugType": req.debug_type })),
	};
	Service
		.environment
		.ApplicationState
		.Extension
		.ProviderRegistration
		.RegisterProvider(Handle, dto);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://debug/register",
		json!({ "debugType": req.debug_type, "extensionId": req.extension_id }),
	);

	Ok(Response::new(Empty {}))
}

pub async fn StartDebugging(
	Service:&CocoonServiceImpl,
	req:StartDebuggingRequest,
) -> Result<Response<StartDebuggingResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] start_debugging: type={}", req.debug_type);

	let SessionId = format!("debug-{}", SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|D| D.as_millis())
		.unwrap_or(0));

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://debug/start",
		json!({
			"sessionId": SessionId,
			"debugType": req.debug_type,
			"configuration": req.configuration.as_ref().map(|C| json!({
				"name": C.name,
				"type": C.r#type,
				"request": C.request,
			})),
		}),
	);

	Ok(Response::new(StartDebuggingResponse { success:true }))
}

pub async fn StopDebugging(
	Service:&CocoonServiceImpl,
	req:StopDebuggingRequest,
) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] stop_debugging: session={}", req.session_id);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://debug/stop",
		json!({ "sessionId": req.session_id }),
	);

	Ok(Response::new(Empty {}))
}
