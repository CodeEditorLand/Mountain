#![allow(non_snake_case)]

//! WorkingCopy domain handlers for Wind IPC.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

/// Check whether a URI has unsaved changes.
pub async fn handle_working_copy_is_dirty(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Uri = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workingCopy:isDirty requires uri".to_string())?;
	let IsDirty = Runtime.Environment.ApplicationState.Feature.WorkingCopy.IsDirty(Uri);
	Ok(json!(IsDirty))
}

/// Mark a URI as dirty (unsaved) or clean.
pub async fn handle_working_copy_set_dirty(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Uri = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workingCopy:setDirty requires uri".to_string())?;
	let Dirty = Args.get(1).and_then(|V| V.as_bool()).unwrap_or(true);
	Runtime.Environment.ApplicationState.Feature.WorkingCopy.SetDirty(Uri, Dirty);
	Ok(Value::Null)
}

/// Return all URIs that currently have unsaved changes.
pub async fn handle_working_copy_get_all_dirty(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Dirty = Runtime.Environment.ApplicationState.Feature.WorkingCopy.GetAllDirty();
	Ok(json!(Dirty))
}

/// Return the count of resources with unsaved changes.
pub async fn handle_working_copy_get_dirty_count(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Count = Runtime.Environment.ApplicationState.Feature.WorkingCopy.GetDirtyCount();
	Ok(json!(Count))
}
