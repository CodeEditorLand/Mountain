//! Keybinding command router.

use std::sync::Arc;

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::UI::{
		KeybindingAdd::Fn as KeybindingAdd,
		KeybindingEvaluateWhen::Fn as KeybindingEvaluateWhen,
		KeybindingGetAll::Fn as KeybindingGetAll,
		KeybindingLookup::Fn as KeybindingLookup,
		KeybindingRemove::Fn as KeybindingRemove,
		KeybindingResolve::Fn as KeybindingResolve,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Routes keybinding commands. Returns Some(result) for handled commands,
/// None otherwise.
pub(crate) async fn route(
	RunTime:Arc<ApplicationRunTime>,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		"keybinding:add" => {
			dev_log!("keybinding", "{}", command);

			Some(KeybindingAdd(RunTime.clone(), Arguments).await)
		},

		"keybinding:remove" => {
			dev_log!("keybinding", "{}", command);

			Some(KeybindingRemove(RunTime.clone(), Arguments).await)
		},

		"keybinding:lookup" => {
			dev_log!("keybinding", "{}", command);

			Some(KeybindingLookup(RunTime.clone(), Arguments).await)
		},

		"keybinding:getAll" => {
			dev_log!("keybinding", "{}", command);

			Some(KeybindingGetAll(RunTime.clone()).await)
		},

		"keybinding:resolve" => {
			dev_log!("keybinding", "{}", command);

			Some(KeybindingResolve(RunTime.clone(), Arguments).await)
		},

		"keybinding:evaluateWhen" => {
			dev_log!("keybinding", "{}", command);

			Some(KeybindingEvaluateWhen(RunTime.clone(), Arguments).await)
		},

		_ => None,
	}
}
