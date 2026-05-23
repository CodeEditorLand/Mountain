
//! Wire method: `workingCopy:getAllDirty`.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Dirty = RunTime.Environment.ApplicationState.Feature.WorkingCopy.GetAllDirty();

	Ok(json!(Dirty))
}
