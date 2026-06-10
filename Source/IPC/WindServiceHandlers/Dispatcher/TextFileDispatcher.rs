//! TextFile dispatcher.

use serde_json::Value;

<<<<<<< HEAD
use crate::Model::{
=======
use crate::IPC::WindServiceHandlers::Model::{
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867
	TextfileRead::Fn as TextfileRead,
	TextfileSave::Fn as TextfileSave,
	TextfileWrite::Fn as TextfileWrite,
};

/// Dispatches text file commands.
pub async fn dispatch_text_file(
<<<<<<< HEAD
	runtime:&crate::RunTime::ApplicationRunTime::ApplicationRunTime,
=======
	runtime:std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867

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
