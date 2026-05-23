#![allow(non_snake_case)]

//! Pop the previous URI off the back-stack and return it.
//! `None` when the stack is empty (caller should disable the
//! back button).

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Uri = RunTime.Environment.ApplicationState.Feature.NavigationHistory.GoBack();

	Ok(Uri.map(Value::String).unwrap_or(Value::Null))
}
