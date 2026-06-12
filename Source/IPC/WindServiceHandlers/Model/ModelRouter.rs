//! Model + textFile command router.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::Emitter;

use crate::{
	IPC::WindServiceHandlers::{
		Model::{
			ModelClose::Fn as ModelClose,
			ModelGet::Fn as ModelGet,
			ModelGetAll::Fn as ModelGetAll,
			ModelOpen::Fn as ModelOpen,
			ModelUpdateContent::Fn as ModelUpdateContent,
			TextfileRead::Fn as TextfileRead,
			TextfileSave::Fn as TextfileSave,
			TextfileWrite::Fn as TextfileWrite,
		},
		Utilities::JsonValueHelpers::arg_string,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Routes model and textFile commands.
pub(crate) async fn route(
	ApplicationHandle:tauri::AppHandle,

	RunTime:Arc<ApplicationRunTime>,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		"textFile:read" => {
			dev_log!("textfile", "{}", command);

			Some(TextfileRead(RunTime.clone(), Arguments).await)
		},

		"textFile:write" => {
			dev_log!("textfile", "{}", command);

			Some(TextfileWrite(RunTime.clone(), Arguments).await)
		},

		"textFile:save" => Some(TextfileSave(RunTime.clone(), Arguments).await),

		"model:open" => {
			dev_log!("model", "model:open");

			Some(ModelOpen(RunTime.clone(), Arguments).await)
		},

		"model:close" => {
			dev_log!("model", "model:close");

			Some(ModelClose(RunTime.clone(), Arguments).await)
		},

		"model:get" => {
			dev_log!("model", "model:get");

			Some(ModelGet(RunTime.clone(), Arguments).await)
		},

		"model:getAll" => {
			dev_log!("model", "model:getAll");

			Some(ModelGetAll(RunTime.clone()).await)
		},

		// relay open intent to Sky so Monaco loads the document
		"text:open" | "workspace:openTextDocument" => {
			let UriStr = arg_string(&Arguments, 0);

			if !UriStr.is_empty() {
				let _ = ApplicationHandle.emit("sky://window/showTextDocument", json!({ "uri": UriStr }));
			}

			Some(Ok(Value::Null))
		},

		"model:updateContent" => {
			dev_log!("model", "model:updateContent");

			Some(ModelUpdateContent(RunTime.clone(), Arguments).await)
		},

		_ => None,
	}
}
