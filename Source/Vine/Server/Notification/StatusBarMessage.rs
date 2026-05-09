#![allow(non_snake_case)]
//! Cocoon → Mountain `statusBar.message` notification.
//! Emitted when an extension calls `vscode.window.setStatusBarMessage`
//! (one-shot text, optional auto-hide). Canonical channel is
//! `sky://statusbar/set-message`.

use serde_json::{Value, json};

use tauri::Emitter;

use crate::{Vine::Server::MountainVinegRPCService::MountainVinegRPCService, dev_log};

pub async fn StatusBarMessage(Service:&MountainVinegRPCService, Parameter:&Value) {

	let Text = Parameter.get("text").and_then(Value::as_str).unwrap_or("");

	let HideAfter = Parameter.get("hideAfter").and_then(Value::as_u64);

	if let Err(Error) = Service.ApplicationHandle().emit(
		"sky://statusbar/set-message",

		json!({
			"text": Text,
			"hideAfter": HideAfter,
		}),
	) {

		dev_log!(
			"grpc",

			"warn: [MountainVinegRPCService] sky://statusbar/set-message emit failed: {}",

			Error
		);
	}
}
