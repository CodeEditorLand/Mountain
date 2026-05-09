#![allow(non_snake_case, unused_variables)]
//! Lifecycle handlers: phase get / wait / shutdown. Tracks Mountain's
//! four-phase startup (Starting / Ready / Restored / Eventually) so Sky
//! can gate UI installation on lifecycle progress.
//!
//! `LifecycleWhenPhase` currently polls at 100 ms intervals up
//! to 5 s; TODO to replace with a `tokio::sync::Notify` when the
//! lifecycle service grows real broadcast semantics.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::AppHandle;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn LifecycleGetPhase(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Phase = RunTime.Environment.ApplicationState.Feature.Lifecycle.GetPhase();

	Ok(json!(Phase))
}

pub async fn LifecycleWhenPhase(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let RequestedPhase = Arguments.first().and_then(|V| V.as_u64()).unwrap_or(1) as u8;

	let CurrentPhase = RunTime.Environment.ApplicationState.Feature.Lifecycle.GetPhase();

	if CurrentPhase >= RequestedPhase {
		return Ok(Value::Null);
	}

	let mut Retries = 0u8;

	loop {
		tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

		let Phase = RunTime.Environment.ApplicationState.Feature.Lifecycle.GetPhase();

		if Phase >= RequestedPhase || Retries >= 50 {
			break;
		}

		Retries += 1;
	}

	Ok(Value::Null)
}

pub async fn LifecycleRequestShutdown(ApplicationHandle:AppHandle) -> Result<Value, String> {
	ApplicationHandle.exit(0);

	Ok(Value::Null)
}
