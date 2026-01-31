// File: Mountain/Source/Environment/StatusBarProvider.rs
// Role: Implements the `StatusBarProvider` trait for the `MountainEnvironment`.
// Responsibilities:
//   - Handle creating, updating, and removing status bar items and messages.
//   - Orchestrate communication between the `Cocoon` sidecar and the `Sky`
//     frontend.
//   - Store status bar state in `ApplicationState` and push updates to the UI.
//   - Manage status bar item ordering and priority system.
//   - Handle dynamic tooltip resolution via sidecar callbacks.
//   - Support status bar item visibility management.
//   - Handle left/right alignment of status bar items.
//
// TODOs:
//   - Implement status bar priority ordering system
//   - Add status bar alignment (left/right) support
//   - Implement status bar item visibility toggle
//   - Support status bar item compact mode
//   - Add status bar item background color support
//   - Implement status bar item grouping
//   - Support status bar item command registration
//   - Add status bar item accessibility (ARIA labels)
//   - Implement status bar item hover actions
//   - Support status bar widget contribution points
//   - Add status bar item animation support
//   - Implement status bar item context menu
//   - Add status bar configuration persistence
//
// Inspired by VSCode's status bar service which:
// - Uses IStatusbarEntryPriority for item ordering
// - Supports StatusbarAlignment (Left/Right)
// - Provides dynamic tooltip resolution
// - Manages entry visibility overrides
// - Supports status bar item compact mode
// - Handles status bar item grouping
//
// ## Status Bar Priority System
//
// The priority determines the order of items within their alignment group:
// - Higher priority values appear before lower priority values
// - Left alignment: Items arranged from left to right by descending priority
// - Right alignment: Items arranged from right to left by descending priority
// - Default priority is 0 for items without explicit priority
// - Primary items typically use priority 100-1000
// - Secondary items typically use priority 10-99
//
// ## Status Bar Item Types
//
// 1. **Persistent Items**: Long-lived items (e.g., branch indicator, language indicator)
// 2. **Transient Messages**: Temporary notifications that auto-dismiss
// 3. **Dynamic Items**: Items with computed values (e.g., error count, position)

//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//!
//! # StatusBarProvider Implementation
//!
//! Implements the `StatusBarProvider` trait for the `MountainEnvironment`. This
//! provider handles creating, updating, and removing status bar items, and
//! orchestrates communication between the `Cocoon` sidecar and the `Sky`
//! frontend.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider},
	StatusBar::{DTO::StatusBarEntryDTO::StatusBarEntryDTO, StatusBarProvider::StatusBarProvider},
};
use async_trait::async_trait;
use log::info;
use serde_json::{Value, json};
use tauri::Emitter;

use super::{MountainEnvironment::MountainEnvironment, Utility};

#[async_trait]
impl StatusBarProvider for MountainEnvironment {
	/// Creates a new status bar entry or updates an existing one.
	async fn SetStatusBarEntry(&self, Entry:StatusBarEntryDTO) -> Result<(), CommonError> {
		info!("[StatusBarProvider] Setting entry: {}", Entry.EntryIdentifier);

		let mut ItemsGuard = self
			.ApplicationState
			.ActiveStatusBarItems
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		ItemsGuard.insert(Entry.EntryIdentifier.clone(), Entry.clone());

		drop(ItemsGuard);

		self.ApplicationHandle
			.emit("sky://statusbar/set-entry", Entry)
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	/// Removes a status bar item from the UI.
	async fn DisposeStatusBarEntry(&self, EntryIdentifier:String) -> Result<(), CommonError> {
		info!("[StatusBarProvider] Disposing entry: {}", EntryIdentifier);

		self.ApplicationState
			.ActiveStatusBarItems
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.remove(&EntryIdentifier);

		self.ApplicationHandle
			.emit("sky://statusbar/dispose-entry", json!({ "EntryIdentifier": EntryIdentifier }))
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	/// Shows a temporary message in the status bar.
	async fn SetStatusBarMessage(&self, MessageIdentifier:String, Text:String) -> Result<(), CommonError> {
		info!("[StatusBarProvider] Setting status message '{}': {}", MessageIdentifier, Text);

		self.ApplicationHandle
			.emit("sky://statusbar/set-message", json!({ "id": MessageIdentifier, "text": Text }))
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	/// Disposes of a temporary status bar message.
	async fn DisposeStatusBarMessage(&self, MessageIdentifier:String) -> Result<(), CommonError> {
		info!("[StatusBarProvider] Disposing status message '{}'", MessageIdentifier);

		self.ApplicationHandle
			.emit("sky://statusbar/dispose-message", json!({ "id": MessageIdentifier }))
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}

	/// Resolves a dynamic tooltip by making a reverse call to the extension
	/// host.
	async fn ProvideTooltip(&self, EntryIdentifier:String) -> Result<Option<Value>, CommonError> {
		info!("[StatusBarProvider] Providing dynamic tooltip for entry: {}", EntryIdentifier);

		let IPCProvider:Arc<dyn IPCProvider> = self.Require();

		// This is a "reverse" call, where the host needs data from the sidecar.
		let RPCMethod = format!("{}$ProvideStatusbarTooltip", ProxyTarget::ExtHostStatusBar.GetTargetPrefix());

		let RPCResponse = IPCProvider
			.SendRequestToSideCar("cocoon-main".to_string(), RPCMethod, json!([EntryIdentifier]), 5000)
			.await?;

		// If the response is null or fails to parse, we gracefully return None.
		Ok(serde_json::from_value(RPCResponse).unwrap_or(None))
	}
}
