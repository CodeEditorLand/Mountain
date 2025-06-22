//! # Keybinding Commands
//!
//! Defines the specific Tauri command handlers for Keybinding data requests
//! that originate from the `Sky` frontend UI.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{Environment::Requires::Requires, Keybinding::KeybindingProvider::KeybindingProvider};
use serde_json::Value;
use tauri::{AppHandle, Manager, command};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime as MountainRunTime;

#[command]
pub async fn GetResolvedKeybinding(app_handle:AppHandle) -> Result<Value, String> {
	log::debug!("[Keybinding Command] Getting resolved keybindings for UI.");
	let runtime = app_handle.state::<Arc<MountainRunTime>>().inner().clone();
	let provider:Arc<dyn KeybindingProvider> = runtime.Environment.Require();

	provider.GetResolvedKeybinding().await.map_err(|e| e.to_string())
}
