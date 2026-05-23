
//! Wipe both the back- and forward-stacks. Issued on workspace
//! close / reload so a stale URL list doesn't leak into the next
//! session.

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	RunTime.Environment.ApplicationState.Feature.NavigationHistory.Clear();

	Ok(Value::Null)
}
