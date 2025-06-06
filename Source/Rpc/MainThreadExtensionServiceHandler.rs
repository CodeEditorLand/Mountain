
// Defines the RPC handler for notifications from the sidecar regarding
// extension lifecycle events (activation, errors, etc.).

use std::sync::Arc;

use log::{debug, info, warn};
use serde_json::Value;
use tauri::{AppHandle, Manager, State, Wry};

use crate::Handlers::{self, ErrorUtils};
use crate::Runtime::AppRuntime; // May not be needed if all logic is in Handlers

#[derive(Clone)]
pub struct MainThreadExtensionServiceHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	// Runtime might not be needed if all logic is in Handlers::ExtensionStatus
	// pub Runtime: Arc<AppRuntime>,
}

impl MainThreadExtensionServiceHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry> /* , Runtime: Arc<AppRuntime> */) -> Self {
		Self { ApplicationHandle /* , Runtime */ }
	}

	// Note: These methods were notifications in the original `track.rs`
	// and are kept as such. They might not directly map to gRPC request/response
	// if the gRPC service defines them as unary calls expecting an Empty response.
	// For gRPC, they would likely be simple async fn that return Result<Value,
	// String> where Value is Null for success.

	/// Handles the notification that an extension is about to be activated.
	pub async fn OnWillActivateExtension(&self, ArgumentArrayValue:Value) -> Result<Value, String> {
		warn!(
			"[Rpc MainThreadExtensionService] OnWillActivateExtension (DTO via fallback). Argument: {:?}",
			ArgumentArrayValue
		);
		Handlers::ExtensionStatus::HandleExtensionHostStatusNotification(
			self.ApplicationHandle.clone(),
			"$onWillActivateExtension", // Method name used by the handler
			ArgumentArrayValue,
		)
		.await
	}

	/// Handles the notification that an extension has successfully activated.
	pub async fn OnDidActivateExtension(&self, ArgumentArrayValue:Value) -> Result<Value, String> {
		warn!(
			"[Rpc MainThreadExtensionService] OnDidActivateExtension (DTO via fallback). Argument: {:?}",
			ArgumentArrayValue
		);
		Handlers::ExtensionStatus::HandleExtensionHostStatusNotification(
			self.ApplicationHandle.clone(),
			"$onDidActivateExtension", // Method name used by the handler
			ArgumentArrayValue,
		)
		.await
	}

	/// Handles the notification that an error occurred during extension
	/// activation.
	pub async fn OnExtensionActivationError(&self, ArgumentArrayValue:Value) -> Result<Value, String> {
		warn!(
			"[Rpc MainThreadExtensionService] OnExtensionActivationError (DTO via fallback). Argument: {:?}",
			ArgumentArrayValue
		);
		Handlers::ExtensionStatus::HandleExtensionHostStatusNotification(
			self.ApplicationHandle.clone(),
			"$onExtensionActivationError", // Method name used by the handler
			ArgumentArrayValue,
		)
		.await
	}

	/// Handles the notification of a runtime error within an extension.
	pub async fn OnExtensionRuntimeError(&self, ArgumentArrayValue:Value) -> Result<Value, String> {
		warn!(
			"[Rpc MainThreadExtensionService] OnExtensionRuntimeError (DTO via fallback). Argument: {:?}",
			ArgumentArrayValue
		);
		Handlers::ExtensionStatus::HandleExtensionHostStatusNotification(
			self.ApplicationHandle.clone(),
			"$onExtensionRuntimeError", // Method name used by the handler
			ArgumentArrayValue,
		)
		.await
	}
}
