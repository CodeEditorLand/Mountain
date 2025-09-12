//! # Keybinding Commands
//!
//! Defines the specific Tauri command handlers for Keybinding data requests
//! that originate from the `Sky` frontend UI.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{Environment::Requires::Requires, Keybinding::KeybindingProvider::KeybindingProvider};
use serde_json::Value;
use tauri::{AppHandle, Manager, Wry, command};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime as MountainRunTime;

#[command]
pub async fn GetResolvedKeybinding(ApplicationHandle:AppHandle<Wry>) -> Result<Value, String> {
	log::debug!("[Keybinding Command] Getting resolved keybindings for UI.");

	let RunTime = ApplicationHandle.state::<Arc<MountainRunTime>>().inner().clone();

	let Provider:Arc<dyn KeybindingProvider> = RunTime.Environment.Require();

	Provider.GetResolvedKeybinding().await.map_err(|Error| Error.to_string())
}
