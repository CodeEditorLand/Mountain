//! Start a debug session. Mints a session id, emits `sky://debug/start` so
//! the workbench can render the debug toolbar/console.

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};
use ::Vine::Generated::{StartDebuggingRequest, StartDebuggingResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:StartDebuggingRequest,
) -> Result<Response<StartDebuggingResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] start_debugging: type={}", Request.debug_type);

	let SessionIdentifier = format!(
		"debug-{}",
		SystemTime::now().duration_since(UNIX_EPOCH).map(|D| D.as_millis()).unwrap_or(0)
	);

	let _ = Service.environment.ApplicationHandle.emit(
		"sky://debug/start",
		json!({
			"sessionId": SessionIdentifier,
			"debugType": Request.debug_type,
			"configuration": Request.configuration.as_ref().map(|C| json!({
				"name": C.name,
				"type": C.r#type,
				"request": C.request,
			})),
		}),
	);

	Ok(Response::new(StartDebuggingResponse { success:true }))
}
