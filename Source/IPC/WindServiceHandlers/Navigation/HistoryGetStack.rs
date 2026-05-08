#![allow(non_snake_case)]

//! Snapshot of the entire navigation history as a `Vec<String>`.
//! Used by the navigate-history quick-pick (Cmd+Alt+-) which
//! lists every recently-visited file inline.

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn HistoryGetStack(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Stack = RunTime.Environment.ApplicationState.Feature.NavigationHistory.GetStack();

	Ok(Value::Array(Stack.into_iter().map(Value::String).collect()))
}
