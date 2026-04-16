#![allow(non_snake_case)]

//! Decoration domain handlers for Wind IPC.

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

/// Return the decoration (badge, tooltip, color) for a single URI.
pub async fn handle_decorations_get(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Uri = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("decorations:get requires uri".to_string())?;
	let Decoration = Runtime.Environment.ApplicationState.Feature.Decorations.GetDecoration(Uri);
	Ok(Decoration.unwrap_or(Value::Null))
}

/// Return decorations for multiple URIs in a single round-trip.
pub async fn handle_decorations_get_many(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Uris:Vec<String> = Args
		.first()
		.and_then(|V| V.as_array())
		.map(|Arr| Arr.iter().filter_map(|U| U.as_str().map(str::to_owned)).collect())
		.unwrap_or_default();

	let mut Result = serde_json::Map::new();
	for Uri in &Uris {
		if let Some(Decoration) = Runtime.Environment.ApplicationState.Feature.Decorations.GetDecoration(Uri) {
			Result.insert(Uri.clone(), Decoration);
		}
	}
	Ok(Value::Object(Result))
}

/// Register or override the decoration for a URI.
pub async fn handle_decorations_set(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Uri = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("decorations:set requires uri".to_string())?;
	let Decoration = Args.get(1).cloned().unwrap_or(Value::Null);
	Runtime
		.Environment
		.ApplicationState
		.Feature
		.Decorations
		.SetDecoration(Uri, Decoration);
	Ok(Value::Null)
}

/// Remove the decoration for a URI.
pub async fn handle_decorations_clear(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Uri = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("decorations:clear requires uri".to_string())?;
	Runtime.Environment.ApplicationState.Feature.Decorations.ClearDecoration(Uri);
	Ok(Value::Null)
}
