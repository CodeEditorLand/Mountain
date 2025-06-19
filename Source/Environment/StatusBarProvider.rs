//! # StatusBarProvider Implementation
//!
//! Implements the `StatusBarProvider` trait for the `MountainEnvironment`. This
//! provider handles creating, updating, and removing status bar items, and
//! orchestrates communication between the `Cocoon` sidecar and the `Sky`
//! frontend.

use std::sync::Arc;

use Common::{
	Error::CommonError::CommonError,
	IPC::IPCProvider,
	StatusBar::{DTO::StatusBarEntryDTO, StatusBarProvider},
};
use async_trait::async_trait;
use log::info;
use serde_json::{Value, json};
use tauri::Emitter;

use super::{MountainEnvironment::MountainEnvironment, Utility};

#[async_trait]
impl StatusBarProvider for MountainEnvironment {
	/// Creates a new status bar entry or updates an existing one.
	async fn SetEntry(&self, Entry:StatusBarEntryDTO) -> Result<(), CommonError> {
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
	async fn DisposeEntry(&self, EntryIdentifier:String) -> Result<(), CommonError> {
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

	/// Resolves a dynamic tooltip by making a reverse call to the extension
	/// host.
	async fn ProvideTooltip(&self, EntryIdentifier:String) -> Result<Option<Value>, CommonError> {
		info!("[StatusBarProvider] Providing dynamic tooltip for entry: {}", EntryIdentifier);
		let IPCProvider:Arc<dyn IPCProvider> = self.Require();

		// This is a "reverse" call, where the host needs data from the sidecar.
		let RPCResponse = IPCProvider
			.SendRequestToSidecar(
				"cocoon-main".to_string(),
				"$ProvideStatusbarTooltip".to_string(),
				json!([EntryIdentifier]),
				5000, // 5-second timeout
			)
			.await?;

		// If the response is null or fails to parse, we gracefully return None.
		Ok(serde_json::from_value(RPCResponse).unwrap_or(None))
	}
}
