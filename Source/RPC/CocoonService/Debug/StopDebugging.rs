//! Stop an active debug session. Emits `sky://debug/sessionEnd` (NOT
//! `/stop` - Sky listens on the former at `SkyBridge.ts:2234`;
//! `DebugProvider.rs:351` emits the same channel from the lifecycle path).
use serde_json::json;
use tauri::Emitter;
use tonic::{Response, Status};
use ::Vine::Generated::{Empty, StopDebuggingRequest};

use crate::{RPC::CocoonService::CocoonServiceImpl, dev_log};

pub async fn Fn(Service:&CocoonServiceImpl, Request:StopDebuggingRequest) -> Result<Response<Empty>, Status> {
	dev_log!("cocoon", "[CocoonService] stop_debugging: session={}", Request.session_id);

	let _ = Service
		.environment
		.ApplicationHandle
		.emit("sky://debug/sessionEnd", json!({ "sessionId": Request.session_id }));

	Ok(Response::new(Empty {}))
}
