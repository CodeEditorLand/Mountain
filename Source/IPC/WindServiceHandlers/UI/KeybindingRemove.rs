
//! Wire method: `keybinding:remove`.

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let CommandId = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:remove requires commandId".to_string())?;

	RunTime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.RemoveKeybinding(CommandId);

	Ok(Value::Null)
}
