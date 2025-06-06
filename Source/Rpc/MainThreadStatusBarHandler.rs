
// Defines the RPC handler for managing status bar entries, allowing the sidecar
// to display or update information in the application's status bar.

use std::sync::Arc;

use log::{debug, error, info, trace}; // Added error
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Wry};

use crate::Runtime::AppRuntime; // May not be strictly needed if only emitting events
use crate::{
	Handlers::ErrorUtils,
	Rpc::Argument::StatusBar::{DisposeEntryArgument, SetEntryArgument},
};

#[derive(Clone)]
pub struct MainThreadStatusBarHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	// Runtime: Arc<AppRuntime>, // Likely not needed for pure event emission
}

impl MainThreadStatusBarHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry> /* , Runtime: Arc<AppRuntime> */) -> Self {
		Self { ApplicationHandle /* , Runtime */ }
	}

	/// Sets or updates a status bar entry.
	/// Emits a Tauri event `mountain:statusbar_set_entry` with the entry DTO.
	pub async fn SetEntry(&self, Argument:SetEntryArgument) -> Result<Value, String> {
		let EntryIdentifierForLog = Argument
			.EntryDto
			.get("id")
			.and_then(Value::as_str)
			.unwrap_or("unknown_statusbar_entry_id");

		info!(
			"[Rpc MainThreadStatusBarHandler] SetEntry (DTO): Identifier='{}'",
			EntryIdentifierForLog
		);
		trace!("[Rpc MainThreadStatusBarHandler] SetEntry Full DTO: {:?}", Argument.EntryDto);

		if let Err(EmitError) = self.ApplicationHandle.emit("mountain:statusbar_set_entry", Argument.EntryDto) {
			let ErrorMessage = format!(
				"Failed to emit 'mountain:statusbar_set_entry' for Identifier='{}': {}",
				EntryIdentifierForLog, EmitError
			);
			error!("[Rpc MainThreadStatusBarHandler] {}", ErrorMessage);
			// Depending on desired behavior, you might return an error or just log it.
			// For consistency with previous patterns, let's return an error string.
			return Err(ErrorUtils::RpcErrorString(ErrorMessage, Some("EEMIT_STATUSBAR_SET")));
		}
		Ok(Value::Null)
	}

	/// Disposes of (removes) a status bar entry.
	/// Emits a Tauri event `mountain:statusbar_dispose_entry` with the entry
	/// ID.
	pub async fn DisposeEntry(&self, Argument:DisposeEntryArgument) -> Result<Value, String> {
		info!(
			"[Rpc MainThreadStatusBarHandler] DisposeEntry (DTO): Identifier='{}'",
			Argument.EntryIdentifier
		);

		if let Err(EmitError) = self
			.ApplicationHandle
			.emit("mountain:statusbar_dispose_entry", json!({ "id": Argument.EntryIdentifier }))
		{
			let ErrorMessage = format!(
				"Failed to emit 'mountain:statusbar_dispose_entry' for Identifier='{}': {}",
				Argument.EntryIdentifier, EmitError
			);
			error!("[Rpc MainThreadStatusBarHandler] {}", ErrorMessage);
			return Err(ErrorUtils::RpcErrorString(ErrorMessage, Some("EEMIT_STATUSBAR_DISPOSE")));
		}
		Ok(Value::Null)
	}
}
