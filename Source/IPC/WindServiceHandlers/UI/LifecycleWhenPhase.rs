#![allow(unused_variables)]

//! Wire method: `lifecycle:whenPhase`.
//! Awaits `LifecyclePhaseState::PhaseNotify` instead of polling.
//! Each forward phase transition calls `notify_waiters()`, so callers wake
//! exactly when the target phase arrives. Hard cap at 30 s.

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let RequestedPhase = Arguments.first().and_then(|V| V.as_u64()).unwrap_or(1) as u8;

	let Lifecycle = &RunTime.Environment.ApplicationState.Feature.Lifecycle;

	if Lifecycle.GetPhase() >= RequestedPhase {
		return Ok(Value::Null);
	}

	let Notify = Lifecycle.PhaseNotify.clone();

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
