#![allow(non_snake_case)]

//! Write text to a file on disk. Counterpart to `TextfileRead`;
//! does not touch the document registry. Callers wanting Monaco
//! to observe the change should follow up with
//! `ModelUpdateContent` for the same URI.

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn TextfileWrite(_runtime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Path = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "textFile:write requires path as first argument".to_string())?;

	let Content = Arguments.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	tokio::fs::write(Path, Content.as_bytes())
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("textFile:write failed: {}", Error))
}
