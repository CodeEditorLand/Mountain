#![allow(non_snake_case, unused_variables)]

//! Wire method: `decorations:getMany`.
//! Bulk-reads decorations for an array of URIs from the in-memory cache.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Uris:Vec<String> = Arguments
		.first()
		.and_then(|V| V.as_array())
		.map(|Arr| Arr.iter().filter_map(|U| U.as_str().map(str::to_owned)).collect())
		.unwrap_or_default();

	let mut Result = serde_json::Map::new();

	for Uri in &Uris {
		if let Some(Decoration) = RunTime.Environment.ApplicationState.Feature.Decorations.GetDecoration(Uri) {
			Result.insert(Uri.clone(), Decoration);
		}
	}

	Ok(Value::Object(Result))
}
