//! Wire method: `workingCopy:getDirtyCount`.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Count = RunTime.Environment.ApplicationState.Feature.WorkingCopy.GetDirtyCount();

	Ok(json!(Count))
}
