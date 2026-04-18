#![allow(non_snake_case)]

//! TextFile domain handlers for Wind IPC.

use std::sync::Arc;

use serde_json::Value;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

/// Read a text file from disk.
pub async fn handle_textfile_read(_Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Path = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "textFile:read requires path as first argument".to_string())?;

	tokio::fs::read_to_string(Path)
		.await
		.map(Value::String)
		.map_err(|Error| format!("textFile:read failed: {}", Error))
}

/// Write text to a file on disk.
pub async fn handle_textfile_write(_Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Path = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or_else(|| "textFile:write requires path as first argument".to_string())?;
	let Content = Args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	tokio::fs::write(Path, Content.as_bytes())
		.await
		.map(|()| Value::Null)
		.map_err(|Error| format!("textFile:write failed: {}", Error))
}

/// Save a document - forward save intent to Sky frontend.
pub async fn handle_textfile_save(_Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let _Uri = Args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
	dev_log!("vfs", "textFile:save uri={:?}", _Uri);
	Ok(Value::Null)
}
