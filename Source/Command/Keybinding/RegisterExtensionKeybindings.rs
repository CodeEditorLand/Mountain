//! Tauri command - register keybindings contributed by an extension at
//! runtime (post-scan contributions, e.g. from `vscode.commands` setup
//! code). Entries land in `ApplicationState::Feature::Keybindings` tagged
//! with the extension identifier so `UnregisterExtensionKeybindings` can
//! remove exactly this extension's entries, and so
//! `KeybindingProvider::GetResolvedKeybinding` reports them with a
//! `dynamic:<extension>` source.
//!
//! Accepts either an array of `{key, command, when?}` objects or a single
//! such object. Entries missing `key` or `command` are skipped and
//! counted in the response.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Wry, command};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime as Runtime, dev_log};

#[command]
pub async fn RegisterExtensionKeybindings(
	ApplicationHandle:AppHandle<Wry>,

	ExtensionIdentifier:String,

	Keybindings:Value,
) -> Result<Value, String> {
	dev_log!("keybinding", "registering keybindings for extension: {}", ExtensionIdentifier);

	let RunTime = ApplicationHandle.state::<Arc<Runtime>>().inner().clone();

	let Registry = &RunTime.Environment.ApplicationState.Feature.Keybindings;

	let Entries:Vec<Value> = match Keybindings {
		Value::Array(Items) => Items,
		Single @ Value::Object(_) => vec![Single],
		_ => {
			return Err("Keybindings must be an object or an array of objects".to_string());
		},
	};

	let mut Registered = 0usize;

	let mut Skipped = 0usize;

	for Entry in &Entries {
		let Key = Entry.get("key").and_then(Value::as_str);

		let Command = Entry.get("command").and_then(Value::as_str);

		match (Key, Command) {
			(Some(Key), Some(Command)) => {
				let When = Entry.get("when").and_then(Value::as_str).map(str::to_owned);

				Registry.AddKeybindingFromSource(
					Command.to_owned(),
					Key.to_owned(),
					When,
					ExtensionIdentifier.clone(),
				);

				Registered += 1;
			},
			_ => {
				Skipped += 1;
			},
		}
	}

	Ok(json!({ "success": true, "registered": Registered, "skipped": Skipped }))
}
