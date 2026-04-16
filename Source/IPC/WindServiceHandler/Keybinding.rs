#![allow(non_snake_case)]

//! Keybinding domain handlers for Wind IPC.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

/// Register a dynamic keybinding in Mountain's keybinding registry.
pub async fn handle_keybinding_add(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let CommandId = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:add requires commandId".to_string())?
		.to_owned();
	let KeyExpression = Args
		.get(1)
		.and_then(|V| V.as_str())
		.ok_or("keybinding:add requires keybinding".to_string())?
		.to_owned();
	let When = Args.get(2).and_then(|V| V.as_str()).map(str::to_owned);
	Runtime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.AddKeybinding(CommandId, KeyExpression, When);
	Ok(Value::Null)
}

/// Remove all dynamic keybindings for a command.
pub async fn handle_keybinding_remove(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let CommandId = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:remove requires commandId".to_string())?;
	Runtime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.RemoveKeybinding(CommandId);
	Ok(Value::Null)
}

/// Look up the keybinding string for a command.
pub async fn handle_keybinding_lookup(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let CommandId = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("keybinding:lookup requires commandId".to_string())?;
	let Binding = Runtime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.LookupKeybinding(CommandId);
	Ok(Binding.map(|B| json!(B)).unwrap_or(Value::Null))
}

/// Return all registered dynamic keybindings.
pub async fn handle_keybinding_get_all(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let All = Runtime.Environment.ApplicationState.Feature.Keybindings.GetAllKeybindings();
	Ok(json!(All))
}
