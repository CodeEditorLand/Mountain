#![allow(non_snake_case, unused_variables)]
//! Working-copy (dirty-state) handlers. Tracks whether an open URI has
//! unsaved changes; Sky queries this to paint the tab's dot-indicator,
//! gate exit dialogs, and drive "Save All" affordances.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn handle_working_copy_is_dirty(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workingCopy:isDirty requires uri".to_string())?;
	let IsDirty = runtime.Environment.ApplicationState.Feature.WorkingCopy.IsDirty(Uri);
	Ok(json!(IsDirty))
}

pub async fn handle_working_copy_set_dirty(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let Uri = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workingCopy:setDirty requires uri".to_string())?;
	let Dirty = args.get(1).and_then(|V| V.as_bool()).unwrap_or(true);
	runtime.Environment.ApplicationState.Feature.WorkingCopy.SetDirty(Uri, Dirty);
	Ok(Value::Null)
}

pub async fn handle_working_copy_get_all_dirty(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Dirty = runtime.Environment.ApplicationState.Feature.WorkingCopy.GetAllDirty();
	Ok(json!(Dirty))
}

pub async fn handle_working_copy_get_dirty_count(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Count = runtime.Environment.ApplicationState.Feature.WorkingCopy.GetDirtyCount();
	Ok(json!(Count))
}
