#![allow(unused_variables)]
//! Working-copy (dirty-state) handlers. Tracks whether an open URI has
//! unsaved changes; Sky queries this to paint the tab's dot-indicator,
//! gate exit dialogs, and drive "Save All" affordances.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn WorkingCopyIsDirty(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Uri = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workingCopy:isDirty requires uri".to_string())?;

	let IsDirty = RunTime.Environment.ApplicationState.Feature.WorkingCopy.IsDirty(Uri);

	Ok(json!(IsDirty))
}

pub async fn WorkingCopySetDirty(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Uri = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workingCopy:setDirty requires uri".to_string())?;

	let Dirty = Arguments.get(1).and_then(|V| V.as_bool()).unwrap_or(true);

	RunTime.Environment.ApplicationState.Feature.WorkingCopy.SetDirty(Uri, Dirty);

	Ok(Value::Null)
}

pub async fn WorkingCopyGetAllDirty(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Dirty = RunTime.Environment.ApplicationState.Feature.WorkingCopy.GetAllDirty();

	Ok(json!(Dirty))
}

pub async fn WorkingCopyGetDirtyCount(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Count = RunTime.Environment.ApplicationState.Feature.WorkingCopy.GetDirtyCount();

	Ok(json!(Count))
}
