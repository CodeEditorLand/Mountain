#![allow(non_snake_case, unused_variables)]
//! Dynamic keybinding handlers. Extensions' `package.json > contributes.
//! keybindings` is a declarative registry; this surface is for the
//! imperative `keybindings:add/remove/lookup/getAll` IPC path Wind uses
//! for runtime registrations (e.g. palette-installed commands).

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn handle_keybinding_add(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let CommandId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:add requires commandId".to_string())?
		.to_owned();
	let KeyExpression = args
		.get(1)
		.and_then(|V| V.as_str())
		.ok_or("keybinding:add requires keybinding".to_string())?
		.to_owned();
	let When = args.get(2).and_then(|V| V.as_str()).map(str::to_owned);
	runtime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.AddKeybinding(CommandId, KeyExpression, When);
	Ok(Value::Null)
}

pub async fn handle_keybinding_remove(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let CommandId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:remove requires commandId".to_string())?;
	runtime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.RemoveKeybinding(CommandId);
	Ok(Value::Null)
}

pub async fn handle_keybinding_lookup(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let CommandId = args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:lookup requires commandId".to_string())?;
	let Binding = runtime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.LookupKeybinding(CommandId);
	Ok(Binding.map(|B| json!(B)).unwrap_or(Value::Null))
}

pub async fn handle_keybinding_get_all(runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let All = runtime.Environment.ApplicationState.Feature.Keybindings.GetAllKeybindings();
	Ok(json!(All))
}
