// File: Mountain/Source/Environment/StatusBarProvider.rs
// Role: Implements the `StatusBarProvider` trait for the `MountainEnvironment`.
// Responsibilities:
//   - Handle creating, updating, and removing status bar items and messages.
//   - Orchestrate communication between the `Cocoon` sidecar and the `Sky`
//     frontend.
//   - Store status bar state in `ApplicationState` and push updates to the UI.

//! # StatusBarProvider Implementation
//!
//! Implements the `StatusBarProvider` trait for the `MountainEnvironment`. This
//! provider handles creating, updating, and removing status bar items, and
//! orchestrates communication between the `Cocoon` sidecar and the `Sky`
//! frontend.

use std::sync::Arc;

use Common::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	IPC::IPCProvider::IPCProvider,
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

		// Store the latest state of the item.
		ItemsGuard.insert(Entry.EntryIdentifier.clone(), Entry.clone());

		drop(ItemsGuard);

		// Notify the Sky frontend to render or update the item.
		self.ApplicationHandle
			.emit("sky://statusbar/set-entry", Entry)
			.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })
	}

	/// Removes a status bar item from the UI.
	async fn DisposeStatusBarEntry(&self, EntryIdentifier:String) -> Result<(), CommonError> {
		info!("[StatusBarProvider] Disposing entry: {}", EntryIdentifier);

		let mut ItemsGuard = self
			.ApplicationState
			.ActiveStatusBarItems
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		ItemsGuard.remove(&EntryIdentifier);

		drop(ItemsGuard);

		// Notify the Sky frontend to remove the item from the UI.
		self.ApplicationHandle
			.emit("sky://statusbar/dispose-entry", json!({ "EntryIdentifier": EntryIdentifier }))
			.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })
	}

	/// Shows a temporary message in the status bar.
	async fn SetStatusBarMessage(&self, MessageIdentifier:String, Text:String) -> Result<(), CommonError> {
		info!("[StatusBarProvider] Setting status message '{}': {}", MessageIdentifier, Text);

		self.ApplicationHandle
			.emit("sky://statusbar/set-message", json!({ "id": MessageIdentifier, "text": Text }))
			.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })
	}

	/// Disposes of a temporary status bar message.
	async fn DisposeStatusBarMessage(&self, MessageIdentifier:String) -> Result<(), CommonError> {
		info!("[StatusBarProvider] Disposing status message '{}'", MessageIdentifier);

		self.ApplicationHandle
			.emit("sky://statusbar/dispose-message", json!({ "id": MessageIdentifier }))
			.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })
	}

	/// Resolves a dynamic tooltip by making a reverse call to the extension
	/// host.
	async fn ProvideTooltip(&self, EntryIdentifier:String) -> Result<Option<Value>, CommonError> {
		info!("[StatusBarProvider] Providing dynamic tooltip for entry: {}", EntryIdentifier);

		let IPCProvider:Arc<dyn IPCProvider> = self.Require();

		// This is a "reverse" call, where the host needs data from the sidecar.
		let RPCResponse = IPCProvider
			.SendRequestToSideCar(
				"cocoon-main".to_string(),
				"$ProvideStatusbarTooltip".to_string(),
				json!([EntryIdentifier]),
				// 5-second timeout
				5000,
			)
			.await?;

		// If the response is null or fails to parse, we gracefully return None.
		Ok(serde_json::from_value(RPCResponse).unwrap_or(None))
	}
}
