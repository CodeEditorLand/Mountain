//! Output channel command router.

use std::sync::Arc;

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::Output::{
		OutputAppend::Fn as OutputAppend,
		OutputAppendLine::Fn as OutputAppendLine,
		OutputClear::Fn as OutputClear,
		OutputCreate::Fn as OutputCreate,
		OutputDispose::Fn as OutputDispose,
		OutputReplace::Fn as OutputReplace,
		OutputShow::Fn as OutputShow,
	},
	dev_log,
};

/// Routes output channel commands.
pub(crate) async fn route(
	ApplicationHandle:tauri::AppHandle,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		"output:create" => Some(OutputCreate(ApplicationHandle.clone(), Arguments).await),

		"output:append" => {
			dev_log!("output", "{}", command);

			Some(OutputAppend(ApplicationHandle.clone(), Arguments).await)
		},

		"output:appendLine" => {
			dev_log!("output", "{}", command);

			Some(OutputAppendLine(ApplicationHandle.clone(), Arguments).await)
		},

		"output:clear" => {
			dev_log!("output", "{}", command);

			Some(OutputClear(ApplicationHandle.clone(), Arguments).await)
		},

		"output:show" => {
			dev_log!("output", "{}", command);

			Some(OutputShow(ApplicationHandle.clone(), Arguments).await)
		},

		"output:replace" => {
			dev_log!("output", "{}", command);

			Some(OutputReplace(ApplicationHandle.clone(), Arguments).await)
		},

		"output:dispose" => {
			dev_log!("output", "{}", command);

			Some(OutputDispose(ApplicationHandle.clone(), Arguments).await)
		},

		_ => None,
	}
}
