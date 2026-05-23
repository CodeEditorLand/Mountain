//! Wire method: `keybinding:add`.

use std::sync::Arc;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let CommandId = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:add requires commandId".to_string())?
		.to_owned();

	let KeyExpression = Arguments
		.get(1)
		.and_then(|V| V.as_str())
		.ok_or("keybinding:add requires keybinding".to_string())?
		.to_owned();

	let When = Arguments.get(2).and_then(|V| V.as_str()).map(str::to_owned);

	RunTime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.AddKeybinding(CommandId, KeyExpression, When);

	Ok(Value::Null)
}
