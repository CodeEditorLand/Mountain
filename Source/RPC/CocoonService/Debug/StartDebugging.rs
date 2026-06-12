//! Start a debug session. Mints a session id, emits `sky://debug/start` so
//! the workbench can render the debug toolbar/console.
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};
use ::Vine::Generated::{StartDebuggingRequest, StartDebuggingResponse};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

/// Monotonic counter for debug session identifiers. Process-unique even
/// when multiple sessions start within the same millisecond.
static NEXT_DEBUG_SESSION_ID:AtomicU64 = AtomicU64::new(1);

pub async fn Fn(
	Service:&CocoonServiceImpl,

	Request:StartDebuggingRequest,
) -> Result<Response<StartDebuggingResponse>, Status> {
	dev_log!("cocoon", "[CocoonService] start_debugging: type={}", Request.debug_type);

	let SessionIdentifier = format!("debug-{}", NEXT_DEBUG_SESSION_ID.fetch_add(1, Ordering::Relaxed));

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
