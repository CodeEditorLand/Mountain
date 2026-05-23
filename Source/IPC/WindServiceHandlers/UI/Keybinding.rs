//! Dynamic keybinding handlers. Extensions' `package.json > contributes.
//! keybindings` is a declarative registry; this surface is for the
//! imperative `keybindings:add/remove/lookup/getAll` IPC path Wind uses
//! for RunTime registrations (e.g. palette-installed commands).

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn KeybindingAdd(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
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

pub async fn KeybindingRemove(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
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

pub async fn KeybindingLookup(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
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

pub async fn KeybindingGetAll(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let All = RunTime.Environment.ApplicationState.Feature.Keybindings.GetAllKeybindings();

	Ok(json!(All))
}
