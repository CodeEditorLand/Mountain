// ============================================================================
// File: Mountain/Source/Command/Keybinding.rs
// ============================================================================
// # Keybinding Commands
//! Defines the specific Tauri command handlers for Keybinding data requests
//! that originate from the `Sky` frontend UI.
//!
//! ## Key Features:
//! - Keybinding resolution and dispatch
//! - Keybinding conflict detection
//! - User keybinding preferences
//! - Extension keybinding registration
//!
//! ## VSCode Reference:
//! - vs/workbench/services/keybinding/browser/keybindingService.ts
//! - vs/platform/keybinding/common/keybindingResolver.ts
// ============================================================================

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Keybinding::KeybindingProvider::KeybindingProvider};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Wry, command};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime as MountainRunTime;

#[command]
pub async fn GetResolvedKeybinding(ApplicationHandle:AppHandle<Wry>) -> Result<Value, String> {
	log::debug!("[Keybinding Command] Getting resolved keybindings for UI.");

	let RunTime = ApplicationHandle.state::<Arc<MountainRunTime>>().inner().clone();

	let Provider:Arc<dyn KeybindingProvider> = RunTime.Environment.Require();

	Provider.GetResolvedKeybinding().await.map_err(|Error| Error.to_string())
}

#[command]
pub async fn GetUserKeybindings(ApplicationHandle:AppHandle<Wry>) -> Result<Value, String> {
	log::debug!("[Keybinding Command] Getting user keybindings for UI.");

	let RunTime = ApplicationHandle.state::<Arc<MountainRunTime>>().inner().clone();

	let Provider:Arc<dyn KeybindingProvider> = RunTime.Environment.Require();

	// TODO: Implement retrieval of user keybindings
	Ok(json!({ "keybindings": [] }))
}

#[command]
pub async fn RegisterExtensionKeybindings(
	ApplicationHandle:AppHandle<Wry>,

	ExtensionIdentifier:String,

	Keybindings:Value,
) -> Result<Value, String> {
	log::debug!(
		"[Keybinding Command] Registering keybindings for extension: {}",
		ExtensionIdentifier
	);

	let RunTime = ApplicationHandle.state::<Arc<MountainRunTime>>().inner().clone();

	let Provider:Arc<dyn KeybindingProvider> = RunTime.Environment.Require();

	// TODO: Implement extension keybinding registration
	Ok(json!({ "success": true }))
}

#[command]
pub async fn UnregisterExtensionKeybindings(
	ApplicationHandle:AppHandle<Wry>,

	ExtensionIdentifier:String,
) -> Result<Value, String> {
	log::debug!(
		"[Keybinding Command] Unregistering keybindings for extension: {}",
		ExtensionIdentifier
	);

	let RunTime = ApplicationHandle.state::<Arc<MountainRunTime>>().inner().clone();

	let Provider:Arc<dyn KeybindingProvider> = RunTime.Environment.Require();

	// TODO: Implement extension keybinding unregistration
	Ok(json!({ "success": true }))
}

#[command]
pub async fn CheckKeybindingConflicts(ApplicationHandle:AppHandle<Wry>, Keybinding:String) -> Result<Value, String> {
	log::debug!("[Keybinding Command] Checking conflicts for keybinding: {}", Keybinding);

	let RunTime = ApplicationHandle.state::<Arc<MountainRunTime>>().inner().clone();

	let Provider:Arc<dyn KeybindingProvider> = RunTime.Environment.Require();

	// TODO: Implement keybinding conflict detection
	Ok(json!({ "conflicts": [] }))
}
