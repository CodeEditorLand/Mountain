#![allow(non_snake_case, unused_variables)]

//! Wire method: `keybinding:lookup`.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let CommandId = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:lookup requires commandId".to_string())?;

	let Binding = RunTime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.LookupKeybinding(CommandId);

	Ok(Binding.map(|B| json!(B)).unwrap_or(Value::Null))
}
