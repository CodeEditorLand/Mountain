//! Wire method: `localPty:updateProperty`.
//!
//! Persists terminal property updates (title, shell type) to the feature state
//! and emits `TerminalPropertyChanged` to Sky so the xterm tab label, icon, and
//! shell-integration features update live.

use CommonLibrary::IPC::SkyEvent::SkyEvent;
use serde_json::{Value, json};
use std::sync::Arc;
use tauri::Emitter;

use crate::{
	Environment::MountainEnvironment::MountainEnvironment,
	IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_u64,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

pub(crate) async fn Fn(
	RunTime: Arc<ApplicationRunTime>,
	Arguments: Vec<Value>,
) -> Result<Value, String> {
	let TermId = arg_u64(&Arguments, 0);
	let PropId = arg_u64(&Arguments, 1);
	let PropValue = Arguments.get(2).and_then(Value::as_str).unwrap_or("").to_string();

	if TermId == 0 || PropValue.is_empty() {
		return Ok(Value::Null);
	}

	match PropId {
		// Title (2) or OverrideName (3): persist + emit to Sky.
		2 | 3 => {
			{
				let Guard =
					RunTime.Environment.ApplicationState.Feature.Terminals.ActiveTerminals.lock();
				if let Some(Entry) = Guard.get(&TermId) {
					Entry.lock().Title = PropValue.clone();
				}
			}

			dev_log!(
				"terminal",
				"localPty:updateProperty id={} prop={} title='{}'",
				TermId,
				PropId,
				PropValue
			);

			let _ = RunTime.Environment.ApplicationHandle.emit(
				SkyEvent::TerminalPropertyChanged.AsStr(),
				json!({
					"id": TermId,
					"property": PropId,
					"value": PropValue,
				}),
			);
		},

		// ShellType (5): store only; workbench derives its own icon.
		5 => {
			{
				let Guard =
					RunTime.Environment.ApplicationState.Feature.Terminals.ActiveTerminals.lock();
				if let Some(Entry) = Guard.get(&TermId) {
					Entry.lock().ShellType = Some(PropValue.clone());
				}
			}

			dev_log!(
				"terminal",
				"localPty:updateProperty id={} shell_type='{}'",
				TermId,
				PropValue
			);
		},

		Other => {
			dev_log!(
				"terminal",
				"localPty:updateProperty id={} unknown_prop={} (no-op)",
				TermId,
				Other
			);
		},
	}

	Ok(Value::Null)
}
