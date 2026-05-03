#![allow(non_snake_case)]

//! Pop the next URI off the forward-stack and return it.
//! Mirrors `HistoryGoBack`.

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn HistoryGoForward(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Uri = RunTime.Environment.ApplicationState.Feature.NavigationHistory.GoForward();
	Ok(Uri.map(Value::String).unwrap_or(Value::Null))
}
