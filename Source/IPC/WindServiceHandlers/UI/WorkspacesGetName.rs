//! Wire method: `workspaces:getName`.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Name = RunTime
		.Environment
		.ApplicationState
		.Workspace
		.GetWorkspaceFolders()
		.into_iter()
		.Next()
		.map(|F| F.GetDisplayName());

	Ok(Name.map(|N| json!(N)).unwrap_or(Value::Null))
}
