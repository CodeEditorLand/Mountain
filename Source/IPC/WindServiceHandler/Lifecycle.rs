#![allow(non_snake_case)]

//! Lifecycle domain handlers for Wind IPC.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::AppHandle;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

/// Return the current application lifecycle phase (1-4).
pub async fn handle_lifecycle_get_phase(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Phase = Runtime.Environment.ApplicationState.Feature.Lifecycle.GetPhase();
	Ok(json!(Phase))
}

/// Wait (poll) until the application reaches at least the requested phase.
/// Returns immediately if the phase has already been reached.
pub async fn handle_lifecycle_when_phase(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let RequestedPhase = Args.first().and_then(|V| V.as_u64()).unwrap_or(1) as u8;
	let CurrentPhase = Runtime.Environment.ApplicationState.Feature.Lifecycle.GetPhase();
	if CurrentPhase >= RequestedPhase {
		return Ok(Value::Null);
	}
	// Simple poll with short sleep - production should use a channel/notify
	let mut Retries = 0u8;
	loop {
		tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
		let Phase = Runtime.Environment.ApplicationState.Feature.Lifecycle.GetPhase();
		if Phase >= RequestedPhase || Retries >= 50 {
			break;
		}
		Retries += 1;
	}
	Ok(Value::Null)
}

/// Initiate a graceful application shutdown via Tauri.
pub async fn handle_lifecycle_request_shutdown(AppHandle:AppHandle) -> Result<Value, String> {
	AppHandle.exit(0);
	Ok(Value::Null)
}

/// Advance the application lifecycle phase (1=Starting, 2=Ready, 3=Restored,
/// 4=Eventually) and emit `sky://lifecycle/phaseChanged` so the workbench can
/// gate long-running work on phases. Called by Wind as the workbench finishes
/// initialisation passes.
pub async fn handle_lifecycle_advance_phase(
	AppHandle:AppHandle,
	Runtime:Arc<ApplicationRunTime>,
	Args:Vec<Value>,
) -> Result<Value, String> {
	let NewPhase = Args.first().and_then(|V| V.as_u64()).unwrap_or(1) as u8;
	Runtime
		.Environment
		.ApplicationState
		.Feature
		.Lifecycle
		.AdvanceAndBroadcast(NewPhase, &AppHandle);
	Ok(json!(Runtime.Environment.ApplicationState.Feature.Lifecycle.GetPhase()))
}
