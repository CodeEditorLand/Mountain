//! # EffectCreation
//!
//! Contains the logic for creating `ActionEffect`s by mapping string-based
//! command and RPC method names to their strongly-typed effect constructors in
//! the `Common` crate. This is the central routing table of the application.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{
	self,
	Effect::{ActionEffect::ActionEffect, ApplicationRunTime::ApplicationRunTime},
	Error::CommonError::CommonError,
};
use serde_json::{Value, from_value};
use tauri::{AppHandle, Runtime};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime as MountainRunTime;

type MappedEffect = ActionEffect<Arc<MountainRunTime>, CommonError, Value>;

/// A helper macro to reduce boilerplate when getting and deserializing
/// parameters from a JSON array.
macro_rules! get_param {
	($params:expr, $index:expr, $type:ty) => {
		from_value::<$type>(
			$params
				.get($index)
				.cloned()
				.ok_or_else(|| format!("Missing parameter at index {}", $index))?,
		)
		.map_err(|e| format!("Invalid parameter at index {}: {}", $index, e))
	};
}

/// Creates an `ActionEffect` for a request from any source (frontend or
/// sidecar).
pub fn CreateEffectForRequest<R:Runtime>(
	_ApplicationHandle:&AppHandle<R>,
	Method:&str,
	Parameters:Value,
) -> Result<MappedEffect, String> {
	let ParametersArray = Parameters
		.as_array()
		.ok_or_else(|| format!("Parameters for '{}' must be an array.", Method))?;

	// This is the main RPC-to-Effect mapping. It deserializes the `params`
	// into the arguments required by the corresponding effect constructor.
	let Effect:MappedEffect = match Method {
		// --- Command Effects ---
		"Command.Execute" => {
			let ID = get_param!(ParametersArray, 0, String)?;
			let Args = ParametersArray.get(1).cloned().unwrap_or(Value::Null);
			Common::Command::ExecuteCommand::ExecuteCommand(ID, Args).map(to_value)
		},
		"Command.Register" => {
			let SidecarID = get_param!(ParametersArray, 0, String)?;
			let CommandID = get_param!(ParametersArray, 1, String)?;
			Common::Command::RegisterCommand::RegisterCommand(SidecarID, CommandID).map(to_value)
		},

		// --- FileSystem Read Effects ---
		"FileSystem.ReadFile" => {
			let Path = get_param!(ParametersArray, 0, _)?;
			Common::FileSystem::ReadFile::ReadFile(Path).map(to_value)
		},
		"FileSystem.StatFile" => {
			let Path = get_param!(ParametersArray, 0, _)?;
			Common::FileSystem::StatFile::StatFile(Path).map(to_value)
		},
		"FileSystem.ReadDirectory" => {
			let Path = get_param!(ParametersArray, 0, _)?;
			Common::FileSystem::ReadDirectory::ReadDirectory(Path).map(to_value)
		},

		// --- FileSystem Write Effects ---
		"FileSystem.WriteFile" => {
			let Path = get_param!(ParametersArray, 0, _)?;
			let Content = get_param!(ParametersArray, 1, Vec<u8>)?;
			let Create = get_param!(ParametersArray, 2, bool)?;
			let Overwrite = get_param!(ParametersArray, 3, bool)?;
			Common::FileSystem::WriteFileBytes::WriteFileBytes(Path, Content, Create, Overwrite).map(to_value)
		},
		"FileSystem.Delete" => {
			let Path = get_param!(ParametersArray, 0, _)?;
			let Recursive = get_param!(ParametersArray, 1, bool)?;
			let UseTrash = get_param!(ParametersArray, 2, bool)?;
			Common::FileSystem::Delete::Delete(Path, Recursive, UseTrash).map(to_value)
		},

		// --- UserInterface Effects ---
		"UserInterface.ShowMessage" => {
			let Severity = get_param!(ParametersArray, 0, _)?;
			let Message = get_param!(ParametersArray, 1, String)?;
			let Options = ParametersArray.get(2).cloned().unwrap_or(Value::Null);
			Common::UserInterface::ShowMessage::ShowMessage(Severity, Message, Options).map(to_value)
		},
		"UserInterface.ShowOpenDialog" => {
			let Options = get_param!(ParametersArray, 0, _)?;
			Common::UserInterface::ShowOpenDialog::ShowOpenDialog(Options).map(to_value)
		},

		// ... Add mappings for all other effects here ...
		_ => return Err(format!("No ActionEffect mapping found for method: {}", Method)),
	}?;

	Ok(Effect)
}

/// A helper function to map the output of any effect to a `serde_json::Value`.
fn to_value<TOutput:serde::Serialize, TError, TRunTime>(
	effect:ActionEffect<Arc<TRunTime>, TError, TOutput>,
) -> Result<MappedEffect, String>
where
	TRunTime: ApplicationRunTime<EnvironmentType = crate::Environment::MountainEnvironment::MountainEnvironment>
		+ Send
		+ Sync
		+ 'static, {
	Ok(effect.map(|output| serde_json::to_value(output).unwrap_or(Value::Null)))
}
