#![allow(non_snake_case)]

//! Predicate: is the back-stack non-empty? Drives the
//! enabled-state of the workbench's back-arrow button.

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	Ok(Value::Bool(
		RunTime.Environment.ApplicationState.Feature.NavigationHistory.CanGoBack(),
	))
}
