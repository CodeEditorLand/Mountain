//! Keybinding command dispatcher.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::UI::{
	KeybindingAdd::Fn as KeybindingAdd,
	KeybindingEvaluateWhen::Fn as KeybindingEvaluateWhen,
	KeybindingGetAll::Fn as KeybindingGetAll,
	KeybindingLookup::Fn as KeybindingLookup,
	KeybindingRemove::Fn as KeybindingRemove,
	KeybindingResolve::Fn as KeybindingResolve,
};

/// Dispatches keybinding commands.
///
/// Handled commands:
/// - `keybinding:add`
/// - `keybinding:remove`
/// - `keybinding:lookup`
/// - `keybinding:getAll`
/// - `keybinding:resolve`
/// - `keybinding:evaluateWhen`
pub async fn dispatch_keybinding(
	runtime:std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"keybinding:add" => KeybindingAdd(runtime.clone(), arguments).await,

		"keybinding:remove" => KeybindingRemove(runtime.clone(), arguments).await,

		"keybinding:lookup" => KeybindingLookup(runtime.clone(), arguments).await,

		"keybinding:getAll" => KeybindingGetAll(runtime.clone()).await,

		"keybinding:resolve" => KeybindingResolve(runtime.clone(), arguments).await,

		"keybinding:evaluateWhen" => KeybindingEvaluateWhen(runtime.clone(), arguments).await,

		_ => Err(format!("Unknown keybinding command: {}", command)),
	}
}
