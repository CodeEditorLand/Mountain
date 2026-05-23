#![allow(unused_variables)]

//! Lifecycle handlers: phase get / wait / shutdown.
//!
//! `LifecycleWhenPhase` awaits `LifecyclePhaseState::PhaseNotify` instead
//! of polling at 100 ms intervals. Each forward phase transition calls
//! `notify_waiters()`, so callers wake exactly when the target phase arrives.

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

	let Lifecycle = &RunTime.Environment.ApplicationState.Feature.Lifecycle;

	// Fast path: already at or past the requested phase.
	if Lifecycle.GetPhase() >= RequestedPhase {
		return Ok(Value::Null);
	}

	let Notify = Lifecycle.PhaseNotify.clone();

	// Hard cap at 30 s so a stalled phase never deadlocks the workbench.
	let _ = tokio::time::timeout(std::time::Duration::from_secs(30), async {
		loop {
			Notify.notified().await;

			if Lifecycle.GetPhase() >= RequestedPhase {
				break;
			}
		}
	})
	.await;

	Ok(Value::Null)
}

pub async fn LifecycleRequestShutdown(ApplicationHandle:AppHandle) -> Result<Value, String> {
	ApplicationHandle.exit(0);

	Ok(Value::Null)
}
