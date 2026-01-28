// File: Mountain/Source/Environment/StatusBarProvider.rs
// Role: Implements the `StatusBarProvider` trait for the `MountainEnvironment`.
// Responsibilities:
//   - Handle creating, updating, and removing status bar items and messages.
//   - Orchestrate communication between the `Cocoon` sidecar and the `Sky`
//     frontend.
//   - Store status bar state in `ApplicationState` and push updates to the UI.

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
