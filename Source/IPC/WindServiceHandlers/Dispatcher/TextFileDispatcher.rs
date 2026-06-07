//! TextFile dispatcher.

use serde_json::Value;

use crate::Model::{
	TextfileRead::Fn as TextfileRead,
	TextfileSave::Fn as TextfileSave,
	TextfileWrite::Fn as TextfileWrite,
};

/// Dispatches text file commands.
pub async fn dispatch_text_file(
	runtime:&crate::RunTime::ApplicationRunTime::ApplicationRunTime,

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"textFile:read" => TextfileRead(runtime.clone(), arguments).await,

		"textFile:write" => TextfileWrite(runtime.clone(), arguments).await,

		"textFile:save" => TextfileSave(runtime.clone(), arguments).await,

		_ => Err(format!("Unknown text file command: {}", command)),
	}
}
