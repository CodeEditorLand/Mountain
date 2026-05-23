#![allow(non_snake_case, unused_variables)]

//! Wire method: `workingCopy:setDirty`.

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Uri = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workingCopy:setDirty requires uri".to_string())?;

	let Dirty = Arguments.get(1).and_then(|V| V.as_bool()).unwrap_or(true);

	RunTime.Environment.ApplicationState.Feature.WorkingCopy.SetDirty(Uri, Dirty);

	Ok(Value::Null)
}
