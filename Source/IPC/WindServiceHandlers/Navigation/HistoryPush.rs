#![allow(non_snake_case)]

//! Push a URI onto the navigation history. Called by Wind every
//! time the active editor changes, so the back/forward chain
//! reflects the user's actual jump trail.

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn HistoryPush(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {

	let Uri = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("history:push requires uri".to_string())?
		.to_owned();

	RunTime.Environment.ApplicationState.Feature.NavigationHistory.Push(Uri);

	Ok(Value::Null)
}
