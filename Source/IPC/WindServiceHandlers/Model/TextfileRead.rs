//! Read a text file from disk verbatim. Distinct from
//! `ModelOpen` - this returns the bytes without registering a
//! `DocumentStateDTO`. Used by tooling paths that want raw
//! content (e.g. import resolvers, settings inspectors).

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(_runtime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Path = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "textFile:read requires path as first argument".to_string())?;

	tokio::fs::read_to_string(Path)
		.await
		.map(Value::String)
		.map_err(|Error| format!("textFile:read failed: {}", Error))
}
