//! # StatusBarProvider (Environment)
//!
//! Implements the `StatusBarProvider` trait for the `MountainEnvironment`. This
//! provider handles creating, updating, and removing status bar items, and
//! orchestrates communication between the `Cocoon` sidecar and the `Sky`
//! frontend.

use CommonLibrary::{
	Error::CommonError::CommonError,
	StatusBar::{DTO::StatusBarEntryDTO::StatusBarEntryDTO, StatusBarProvider::StatusBarProvider},
};
use async_trait::async_trait;
use serde_json::Value;

use super::MountainEnvironment::MountainEnvironment;

// Private submodules containing the actual implementation
#[path = "StatusBarProvider/EntryManagement.rs"]
mod EntryManagement;

#[path = "StatusBarProvider/MessageManagement.rs"]
mod MessageManagement;

#[path = "StatusBarProvider/Tooltip.rs"]
mod Tooltip;

#[async_trait]
impl StatusBarProvider for MountainEnvironment {
	/// Creates a new status bar entry or updates an existing one.
	async fn SetStatusBarEntry(&self, entry:StatusBarEntryDTO) -> Result<(), CommonError> {
		EntryManagement::set_status_bar_entry_impl(self, entry).await
	}

	/// Removes a status bar item from the UI.
	async fn DisposeStatusBarEntry(&self, entry_identifier:String) -> Result<(), CommonError> {
		EntryManagement::dispose_status_bar_entry_impl(self, entry_identifier).await
	}

	/// Shows a temporary message in the status bar.
	async fn SetStatusBarMessage(&self, message_identifier:String, text:String) -> Result<(), CommonError> {
		MessageManagement::set_status_bar_message_impl(self, message_identifier, text).await
	}

	/// Disposes of a temporary status bar message.
	async fn DisposeStatusBarMessage(&self, message_identifier:String) -> Result<(), CommonError> {
		MessageManagement::dispose_status_bar_message_impl(self, message_identifier).await
	}

	/// Resolves a dynamic Tooltip by making a reverse call to the extension
	/// host.
	async fn ProvideTooltip(&self, entry_identifier:String) -> Result<Option<Value>, CommonError> {
		Tooltip::provide_tooltip_impl(self, entry_identifier).await
	}
}
