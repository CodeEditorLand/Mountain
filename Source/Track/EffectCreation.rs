//! # EffectCreation
//!
//! Contains the logic for creating `ActionEffect`s by mapping string-based
//! command and RPC method names to their strongly-typed effect constructors in
//! the `Common` crate. This is the central routing table of the application.

#![allow(non_snake_case, non_camel_case_types)]

use std::{future::Future, pin::Pin, sync::Arc};

use Common::{
	self,
	Effect::{ActionEffect::ActionEffect, ApplicationRunTime::ApplicationRunTime as ApplicationRunTimeTrait},
	Error::CommonError::CommonError,
};
use serde_json::{Value, from_value};
use tauri::{AppHandle, Runtime};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime as MountainRunTime;

/// A type alias for a boxed, runnable effect. This is the "type-erased" unit of
/// work.
pub type MappedEffect =
	Box<dyn FnOnce(Arc<MountainRunTime>) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> + Send>;

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

/// A helper that takes a specific `ActionEffect`, boxes it, and returns a
/// closure that can be run by the dispatcher.
fn box_effect<C, O, E>(effect:ActionEffect<Arc<C>, E, O>) -> MappedEffect
where
	C: ?Sized + Send + Sync + 'static,
	O: serde::Serialize + Send + Sync + 'static,
	E: Into<CommonError> + Send + Sync + 'static,
	MountainRunTime: Common::Environment::Requires::Requires<C>, {
	Box::new(move |runtime:Arc<MountainRunTime>| {
		Box::pin(async move {
			let result = runtime.Run(effect).await;
			match result {
				Ok(output) => serde_json::to_value(output).map_err(|e| format!("Serialization failed: {}", e)),
				Err(e) => {
					let common_error:CommonError = e.into();
					Err(common_error.to_string())
				},
			}
		})
	})
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

	let effect = match Method {
		// --- Command Effects ---
		"Command.Execute" => {
			let ID = get_param!(ParametersArray, 0, String)?;
			let Args = ParametersArray.get(1).cloned().unwrap_or(Value::Null);
			box_effect(Common::Command::ExecuteCommand::ExecuteCommand(ID, Args))
		},
		"Command.Register" => {
			let SidecarID = get_param!(ParametersArray, 0, String)?;
			let CommandID = get_param!(ParametersArray, 1, String)?;
			box_effect(Common::Command::RegisterCommand::RegisterCommand(SidecarID, CommandID))
		},

		// --- FileSystem Read Effects ---
		"FileSystem.ReadFile" => {
			let Path = get_param!(ParametersArray, 0, _)?;
			box_effect(Common::FileSystem::ReadFile::ReadFile(Path))
		},
		"FileSystem.StatFile" => {
			let Path = get_param!(ParametersArray, 0, _)?;
			box_effect(Common::FileSystem::StatFile::StatFile(Path))
		},
		"FileSystem.ReadDirectory" => {
			let Path = get_param!(ParametersArray, 0, _)?;
			box_effect(Common::FileSystem::ReadDirectory::ReadDirectory(Path))
		},

		// --- FileSystem Write Effects ---
		"FileSystem.WriteFile" => {
			let Path = get_param!(ParametersArray, 0, _)?;
			let Content = get_param!(ParametersArray, 1, Vec<u8>)?;
			let Create = get_param!(ParametersArray, 2, bool)?;
			let Overwrite = get_param!(ParametersArray, 3, bool)?;
			box_effect(Common::FileSystem::WriteFileBytes::WriteFileBytes(
				Path, Content, Create, Overwrite,
			))
		},
		"FileSystem.Delete" => {
			let Path = get_param!(ParametersArray, 0, _)?;
			let Recursive = get_param!(ParametersArray, 1, bool)?;
			let UseTrash = get_param!(ParametersArray, 2, bool)?;
			box_effect(Common::FileSystem::Delete::Delete(Path, Recursive, UseTrash))
		},

		// --- UserInterface Effects ---
		"UserInterface.ShowMessage" => {
			let Severity = get_param!(ParametersArray, 0, _)?;
			let Message = get_param!(ParametersArray, 1, String)?;
			let Options = ParametersArray.get(2).cloned().unwrap_or(Value::Null);
			box_effect(Common::UserInterface::ShowMessage::ShowMessage(Severity, Message, Options))
		},
		"UserInterface.ShowOpenDialog" => {
			let Options = get_param!(ParametersArray, 0, _)?;
			box_effect(Common::UserInterface::ShowOpenDialog::ShowOpenDialog(Options))
		},

		// ... Add mappings for all other effects here ...
		_ => return Err(format!("No ActionEffect mapping found for method: {}", Method)),
	};

	Ok(effect)
}
