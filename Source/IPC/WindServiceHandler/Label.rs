#![allow(non_snake_case)]

//! Label domain handlers for Wind IPC.

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

/// Resolve a human-readable display label for a URI.
///
/// Args: [uri: string, relative: bool]
/// Returns: string label
pub async fn handle_label_get_uri(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Uri = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("label:getUri requires uri".to_string())?
		.to_owned();

	let Relative = Args.get(1).and_then(|V| V.as_bool()).unwrap_or(false);

	if !Relative {
		let Label = if Uri.starts_with("file://") {
			Uri.trim_start_matches("file://").to_owned()
		} else {
			Uri.clone()
		};
		return Ok(Value::String(Label));
	}

	let WorkspaceRoot = Runtime
		.Environment
		.ApplicationState
		.Workspace
		.GetWorkspaceFolders()
		.into_iter()
		.next()
		.map(|F| F.URI.to_string())
		.unwrap_or_default();

	let RawPath = if Uri.starts_with("file://") {
		Uri.trim_start_matches("file://").to_owned()
	} else {
		Uri.clone()
	};

	let RootPath = if WorkspaceRoot.starts_with("file://") {
		WorkspaceRoot.trim_start_matches("file://").to_owned()
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

/// Return the display label for the current workspace root folder.
pub async fn handle_label_get_workspace(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Label = Runtime
		.Environment
		.ApplicationState
		.Workspace
		.GetWorkspaceFolders()
		.into_iter()
		.next()
		.map(|F| {
			if !F.Name.is_empty() {
				F.Name
			} else {
				F.URI
					.path_segments()
					.and_then(|mut S| S.next_back())
					.map(|S| S.to_owned())
					.unwrap_or_else(|| F.URI.to_string())
			}
		})
		.unwrap_or_default();

	Ok(Value::String(Label))
}

/// Return only the basename (filename + extension) of a URI.
pub async fn handle_label_get_base(Args:Vec<Value>) -> Result<Value, String> {
	let Uri = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("label:getBase requires uri".to_string())?;

	let Base = Uri.split('/').next_back().unwrap_or(Uri);
	Ok(Value::String(Base.to_owned()))
}
