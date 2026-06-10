//! Model command dispatcher.

use serde_json::Value;

<<<<<<< HEAD
use crate::Model::{
=======
use crate::IPC::WindServiceHandlers::Model::{
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867
	ModelClose::Fn as ModelClose,
	ModelGet::Fn as ModelGet,
	ModelGetAll::Fn as ModelGetAll,
	ModelOpen::Fn as ModelOpen,
	ModelUpdateContent::Fn as ModelUpdateContent,
	TextfileRead::Fn as TextfileRead,
	TextfileSave::Fn as TextfileSave,
	TextfileWrite::Fn as TextfileWrite,
};

/// Dispatches model commands.
///
/// Handled commands:
/// - `model:open`
/// - `model:close`
/// - `model:get`
/// - `model:getAll`
/// - `model:updateContent`
/// - `textFile:read`
/// - `textFile:write`
/// - `textFile:save`
pub async fn dispatch_model(
<<<<<<< HEAD
	runtime:&crate::RunTime::ApplicationRunTime::ApplicationRunTime,
=======
	runtime:std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"model:open" => ModelOpen(runtime.clone(), arguments).await,

		"model:close" => ModelClose(runtime.clone(), arguments).await,

		"model:get" => ModelGet(runtime.clone(), arguments).await,

		"model:getAll" => ModelGetAll(runtime.clone()).await,

		"model:updateContent" => ModelUpdateContent(runtime.clone(), arguments).await,

		"textFile:read" => TextfileRead(runtime.clone(), arguments).await,

		"textFile:write" => TextfileWrite(runtime.clone(), arguments).await,

		"textFile:save" => TextfileSave(runtime.clone(), arguments).await,

		_ => Err(format!("Unknown model command: {}", command)),
	}
}
