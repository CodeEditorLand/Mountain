#![allow(non_snake_case)]

//! Display label for the current workspace's root folder.
//! Prefers the explicit `Name` if the user set one
//! (`.code-workspace`'s `name` field); otherwise falls back to
//! the trailing path segment of the URI.

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Label = RunTime
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
