//! # EffectCreation
//!
//! Contains the logic for creating `ActionEffect`s by mapping string-based
//! command and RPC method names to their strongly-typed effect constructors in
//! the `Common` crate. This is the central routing table of the application.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{
	Effect::{ActionEffect::ActionEffect, ApplicationRunTime::ApplicationRunTime as ApplicationRunTimeTrait},
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

/// A helper function to erase the specific capability type of an effect,
/// mapping it to a generic effect that can be run by the dispatcher.
fn erase_and_map_effect<C, O, E>(effect:ActionEffect<Arc<C>, E, O>) -> MappedEffect
where
	C: ?Sized + Send + Sync + 'static,
	O: serde::Serialize + Send + 'static,
	E: Into<CommonError> + Send + Sync + 'static,
	MountainRunTime: Common::Environment::Requires::Requires<C>, {
	let mapped_effect = effect.map(|output| serde_json::to_value(output).unwrap_or(Value::Null));

	ActionEffect::New(Arc::new(move |runtime:Arc<MountainRunTime>| {
		let effect_clone = mapped_effect.clone();
		Box::pin(async move { runtime.Run(effect_clone).await.map_err(|e| e.into()) })
	}))
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
	let effect:MappedEffect = match Method {
		// --- Command Effects ---
		"Command.Execute" => {
			let ID = get_param!(ParametersArray, 0, String)?;
			let Args = ParametersArray.get(1).cloned().unwrap_or(Value::Null);
			let specific_effect = Common::Command::ExecuteCommand::ExecuteCommand(ID, Args);
			erase_and_map_effect(specific_effect)
		},
		"Command.Register" => {
			let SidecarID = get_param!(ParametersArray, 0, String)?;
			let CommandID = get_param!(ParametersArray, 1, String)?;
			let specific_effect = Common::Command::RegisterCommand::RegisterCommand(SidecarID, CommandID);
			erase_and_map_effect(specific_effect)
		},

		// --- FileSystem Read Effects ---
		"FileSystem.ReadFile" => {
			let Path = get_param!(ParametersArray, 0, _)?;
			let specific_effect = Common::FileSystem::ReadFile::ReadFile(Path);
			erase_and_map_effect(specific_effect)
		},
		"FileSystem.StatFile" => {
			let Path = get_param!(ParametersArray, 0, _)?;
			let specific_effect = Common::FileSystem::StatFile::StatFile(Path);
			erase_and_map_effect(specific_effect)
		},
		"FileSystem.ReadDirectory" => {
			let Path = get_param!(ParametersArray, 0, _)?;
			let specific_effect = Common::FileSystem::ReadDirectory::ReadDirectory(Path);
			erase_and_map_effect(specific_effect)
		},

		// --- FileSystem Write Effects ---
		"FileSystem.WriteFile" => {
			let Path = get_param!(ParametersArray, 0, _)?;
			let Content = get_param!(ParametersArray, 1, Vec<u8>)?;
			let Create = get_param!(ParametersArray, 2, bool)?;
			let Overwrite = get_param!(ParametersArray, 3, bool)?;
			let specific_effect = Common::FileSystem::WriteFileBytes::WriteFileBytes(Path, Content, Create, Overwrite);
			erase_and_map_effect(specific_effect)
		},
		"FileSystem.Delete" => {
			let Path = get_param!(ParametersArray, 0, _)?;
			let Recursive = get_param!(ParametersArray, 1, bool)?;
			let UseTrash = get_param!(ParametersArray, 2, bool)?;
			let specific_effect = Common::FileSystem::Delete::Delete(Path, Recursive, UseTrash);
			erase_and_map_effect(specific_effect)
		},

		// --- UserInterface Effects ---
		"UserInterface.ShowMessage" => {
			let Severity = get_param!(ParametersArray, 0, _)?;
			let Message = get_param!(ParametersArray, 1, String)?;
			let Options = ParametersArray.get(2).cloned().unwrap_or(Value::Null);
			let specific_effect = Common::UserInterface::ShowMessage::ShowMessage(Severity, Message, Options);
			erase_and_map_effect(specific_effect)
		},
		"UserInterface.ShowOpenDialog" => {
			let Options = get_param!(ParametersArray, 0, _)?;
			let specific_effect = Common::UserInterface::ShowOpenDialog::ShowOpenDialog(Options);
			erase_and_map_effect(specific_effect)
		},

		// ... Add mappings for all other effects here ...
		_ => return Err(format!("No ActionEffect mapping found for method: {}", Method)),
	};

	Ok(effect)
}
