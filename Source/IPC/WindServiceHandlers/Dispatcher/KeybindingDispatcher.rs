//! Keybinding command dispatcher.

use serde_json::Value;

<<<<<<< HEAD
use crate::UI::{
=======
use crate::IPC::WindServiceHandlers::UI::{
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867
	KeybindingAdd::Fn as KeybindingAdd,
	KeybindingGetAll::Fn as KeybindingGetAll,
	KeybindingLookup::Fn as KeybindingLookup,
	KeybindingRemove::Fn as KeybindingRemove,
};

/// Dispatches keybinding commands.
///
/// Handled commands:
/// - `keybinding:add`
/// - `keybinding:remove`
/// - `keybinding:lookup`
/// - `keybinding:getAll`
pub async fn dispatch_keybinding(
<<<<<<< HEAD
	runtime:&crate::RunTime::ApplicationRunTime::ApplicationRunTime,
=======
	runtime:std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"keybinding:add" => KeybindingAdd(runtime.clone(), arguments).await,

		"keybinding:remove" => KeybindingRemove(runtime.clone(), arguments).await,

		"keybinding:lookup" => KeybindingLookup(runtime.clone(), arguments).await,

		"keybinding:getAll" => KeybindingGetAll(runtime.clone()).await,

		_ => Err(format!("Unknown keybinding command: {}", command)),
	}
}
