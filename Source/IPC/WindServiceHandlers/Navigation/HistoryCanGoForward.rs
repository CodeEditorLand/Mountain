#![allow(non_snake_case)]

//! Predicate: is the forward-stack non-empty? Twin of
//! `HistoryCanGoBack`.

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn HistoryCanGoForward(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {

	Ok(Value::Bool(
		RunTime.Environment.ApplicationState.Feature.NavigationHistory.CanGoForward(),
	))
}
