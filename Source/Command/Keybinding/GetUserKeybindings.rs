//! Tauri command - return user-defined keybinding overrides. Stub
//! returns an empty array; pending persistence layer wired through
//! `KeybindingProvider`.
//!
//! ## Planned
//!
//! Hydrate from ApplicationState, including command id, chord, when
//! clause, source extension, and conflict information for the keyboard
//! shortcuts UI.

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Keybinding::KeybindingProvider::KeybindingProvider};

use serde_json::{Value, json};

use tauri::{AppHandle, Manager, Wry, command};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime as Runtime, dev_log};

#[command]
pub async fn GetUserKeybindings(ApplicationHandle:AppHandle<Wry>) -> Result<Value, String> {

	dev_log!("keybinding", "getting user keybindings for UI");

	let RunTime = ApplicationHandle.state::<Arc<Runtime>>().inner().clone();

	let _Provider:Arc<dyn KeybindingProvider> = RunTime.Environment.Require();

	Ok(json!({ "keybindings": [] }))
}
