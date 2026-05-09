#![allow(non_snake_case)]

//! Close a text model. Drops the entry from
//! `ApplicationState.Feature.Documents`. Idempotent - closing
//! an already-closed URI is a no-op.

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn ModelClose(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Uri = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("model:close requires uri".to_string())?;

	RunTime.Environment.ApplicationState.Feature.Documents.Remove(Uri);

	Ok(Value::Null)
}
