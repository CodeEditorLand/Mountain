// @module EffectCreation
// @description Contains the logic for creating `ActionEffect`s by mapping
// string-based command and RPC method names to their strongly-typed effect
// constructors in the `Common` crate.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{
	effect::{ActionEffect, ApplicationRunTime},
	error::CommonError,
};
use log::warn;
use serde_json::Value;
use tauri::{AppHandle, Runtime};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime as MountainRunTime;

type Effect = ActionEffect<Arc<MountainRunTime>, CommonError, Value>;

// Creates an `ActionEffect` for a command invoked from the frontend.
pub fn CreateEffectForFrontendCommand<R:Runtime>(
	_app_handle:&AppHandle<R>,
	command:&str,
	argument:Value,
) -> Result<Effect, String> {
	// The frontend typically sends commands with simpler, self-contained arguments.
	// This function maps the command string to the appropriate effect constructor.
	// For now, many commands will pass through to the sidecar effect creator.
	warn!(
		"[EffectCreation] Frontend command '{}' is being routed through sidecar effect creation. This may be \
		 simplified in the future.",
		command
	);
	CreateEffectForSidecarRequest(command, &argument)
}

// Creates an `ActionEffect` for an RPC request invoked from a sidecar.
pub fn CreateEffectForSidecarRequest(method:&str, params:&Value) -> Result<Effect, String> {
	// This is the main RPC-to-Effect mapping.
	// It deserializes the `params` (usually a JSON array) into the arguments
	// required by the corresponding effect constructor from the `Common` crate.
	let params_array = params
		.as_array()
		.ok_or_else(|| format!("Parameters for '{}' must be an array.", method))?;

	let get_param = |index:usize| {
		params_array
			.get(index)
			.cloned()
			.ok_or_else(|| format!("Missing parameter at index {}", index))
	};

	let effect:ActionEffect<
		Arc<dyn ApplicationRunTime<EnvironmentType = crate::Environment::MountainEnvironment>>,
		CommonError,
		_,
	> = match method {
		// --- Filesystem Effects ---
		"fs.readFile" => {
			let path = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			Common::fs::ReadFile(path).map(map_to_value)
		},
		"fs.writeFile" => {
			let path = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			let content = serde_json::from_value(get_param(1)?).map_err(|e| e.to_string())?;
			// Note: create/overwrite options might need to be passed in a struct
			Common::fs::WriteFile(path, content, true, true).map(map_to_value)
		},
		"fs.stat" => {
			let path = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			Common::fs::StatFile(path).map(map_to_value)
		},
		"fs.readDirectory" => {
			let path = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			Common::fs::ReadDirectory(path).map(map_to_value)
		},
		"fs.createDirectory" => {
			let path = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			Common::fs::CreateDirectory(path, true).map(map_to_value)
		},
		"fs.delete" => {
			let path = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			// Note: recursive/useTrash options might need to be passed
			Common::fs::Delete(path, true, false).map(map_to_value)
		},
		"fs.rename" => {
			let source = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			let target = serde_json::from_value(get_param(1)?).map_err(|e| e.to_string())?;
			Common::fs::Rename(source, target, true).map(map_to_value)
		},
		"fs.copy" => {
			let source = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			let target = serde_json::from_value(get_param(1)?).map_err(|e| e.to_string())?;
			Common::fs::Copy(source, target, true).map(map_to_value)
		},

		// --- Document Effects ---
		"doc.open" => {
			let uri = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			let lang_id = serde_json::from_value(get_param(1)?).map_err(|e| e.to_string())?;
			let content = serde_json::from_value(get_param(2)?).map_err(|e| e.to_string())?;
			Common::document::OpenDocument(uri, lang_id, content).map(map_to_value)
		},
		"doc.save" => {
			let uri = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			Common::document::SaveDocument(uri).map(map_to_value)
		},
		"doc.applyChanges" => {
			let uri = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			let version = serde_json::from_value(get_param(1)?).map_err(|e| e.to_string())?;
			let changes = get_param(2)?;
			Common::document::ApplyDocumentChanges(uri, version, changes).map(map_to_value)
		},

		// --- User Interface Effects ---
		"ui.showMessage" => {
			let severity = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			let message = serde_json::from_value(get_param(1)?).map_err(|e| e.to_string())?;
			let options = get_param(2).ok();
			Common::ui::ShowMessage(severity, message, options).map(map_to_value)
		},
		"ui.showOpenDialog" => {
			let options = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			Common::ui::ShowOpenDialog(options).map(map_to_value)
		},
		"ui.showSaveDialog" => {
			let options = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			Common::ui::ShowSaveDialog(options).map(map_to_value)
		},
		"ui.showQuickPick" => {
			let items = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			let options = serde_json::from_value(get_param(1)?).map_err(|e| e.to_string())?;
			Common::ui::ShowQuickPick(items, options).map(map_to_value)
		},
		"ui.showInputBox" => {
			let options = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			Common::ui::ShowInputBox(options).map(map_to_value)
		},

		// --- Command Effects ---
		"cmd.execute" => {
			let id = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			let args = get_param(1)?;
			Common::command::ExecuteCommand(id, args).map(map_to_value)
		},
		"cmd.register" => {
			let sidecar_id = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			let cmd_id = serde_json::from_value(get_param(1)?).map_err(|e| e.to_string())?;
			Common::command::RegisterCommand(sidecar_id, cmd_id).map(map_to_value)
		},
		"cmd.unregister" => {
			let sidecar_id = serde_json::from_value(get_param(0)?).map_err(|e| e.to_string())?;
			let cmd_id = serde_json::from_value(get_param(1)?).map_err(|e| e.to_string())?;
			Common::command::UnregisterCommand(sidecar_id, cmd_id).map(map_to_value)
		},
		"cmd.getAll" => Common::command::GetAllCommand().map(map_to_value),

		// --- Add other mappings for config, workspace, storage, secrets, etc. ---
		_ => return Err(format!("No ActionEffect mapping found for method: {}", method)),
	};

	Ok(effect)
}

// A helper function to map the output of any effect to a `serde_json::Value`.
fn map_to_value<T:serde::Serialize, E>(
	effect:ActionEffect<Arc<dyn ApplicationRunTime<EnvironmentType = crate::Environment::MountainEnvironment>>, E, T>,
) -> ActionEffect<Arc<MountainRunTime>, E, Value> {
	effect.map(|output| serde_json::to_value(output).unwrap_or(Value::Null))
}
