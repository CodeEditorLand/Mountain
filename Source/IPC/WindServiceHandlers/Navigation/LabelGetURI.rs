//! Resolve a human-readable display label for a URI. Two modes:
//!
//! - `Relative=false`: strip the `file://` scheme and return the raw absolute
//!   path.
//! - `Relative=true`: same, then trim the workspace-folder prefix so the user
//!   sees `Source/main.rs` instead of `/Volumes/.../Mountain/Source/main.rs`.

use std::sync::Arc;

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_bool,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Uri = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("label:getUri requires uri".to_string())?
		.to_owned();

	let Relative = arg_bool(&Arguments, 1);

	if !Relative {
		let Label = if let Some(stripped) = Uri.strip_prefix("file://") {
			stripped.to_owned()
		} else {
			Uri.clone()
		};

		return Ok(Value::String(Label));
	}

	let WorkspaceRoot = RunTime
		.Environment
		.ApplicationState
		.Workspace
		.GetWorkspaceFolders()
		.into_iter()
		.next()
		.map(|F| F.URI.to_string())
		.unwrap_or_default();

	let RawPath = if let Some(stripped) = Uri.strip_prefix("file://") {
		stripped.to_owned()
	} else {
		Uri.clone()
	};

	let RootPath = if let Some(stripped) = WorkspaceRoot.strip_prefix("file://") {
		stripped.to_owned()
	} else {
		WorkspaceRoot
	};

	let Label = if !RootPath.is_empty() && RawPath.starts_with(&RootPath) {
		RawPath[RootPath.len()..].trim_start_matches('/').to_owned()
	} else {
		RawPath
	};

	Ok(Value::String(Label))
}
