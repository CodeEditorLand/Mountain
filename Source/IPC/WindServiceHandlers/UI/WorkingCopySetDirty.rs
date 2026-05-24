//! Wire method: `workingCopy:setDirty`.

use std::sync::Arc;

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::Utilities::JsonValueHelpers::ArgBoolTrue,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Uri = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workingCopy:setDirty requires uri".to_string())?;

	let Dirty = ArgBoolTrue(&Arguments, 1);

	RunTime.Environment.ApplicationState.Feature.WorkingCopy.SetDirty(Uri, Dirty);

	Ok(Value::Null)
}
