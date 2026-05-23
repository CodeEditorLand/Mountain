#![allow(non_snake_case, unused_variables)]

//! Wire method: `workingCopy:isDirty`.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Uri = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workingCopy:isDirty requires uri".to_string())?;

	let IsDirty = RunTime.Environment.ApplicationState.Feature.WorkingCopy.IsDirty(Uri);

	Ok(json!(IsDirty))
}
