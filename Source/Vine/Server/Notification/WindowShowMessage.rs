#![allow(non_snake_case)]
//! Cocoon → Mountain `window.showMessage` notification.
//! Fires when an extension calls `vscode.window.showInformationMessage`
//! / `showWarningMessage` / `showErrorMessage`. Forwards on
//! `sky://notification/show` so the toast stack renders without a
//! round-trip back to Cocoon.
//!
//! Distinct from `Window.ShowMessage` (capitalised) - that variant is
//! a round-trip **request** (sendRequest) awaiting the user's button
//! selection; this one is the fire-and-forget notification form.

use serde_json::Value;
use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn WindowShowMessage(Service:&MountainVinegRPCService, Parameter:&Value) {
	dev_log!(
		"grpc",
		"[WindowShowMessage] message={:?}",
		Parameter.get("message").and_then(Value::as_str).unwrap_or("")
	);
	if let Err(Error) = Service.ApplicationHandle().emit("sky://notification/show", Parameter) {
		dev_log!(
			"grpc",
			"warn: [MountainVinegRPCService] Failed to emit sky://notification/show: {}",
			Error
		);
	}
}
