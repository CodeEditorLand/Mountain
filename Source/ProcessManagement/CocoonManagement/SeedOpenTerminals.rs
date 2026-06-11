//! Seed Cocoon's `__terminals` with every terminal already running so
//! `vscode.window.terminals` is never empty for extensions that read it
//! synchronously in `activate()`.

use std::sync::Arc;

use crate::{Environment::MountainEnvironment::MountainEnvironment, dev_log};

pub(crate) async fn Fn(SideCarId:&str, Environment:&Arc<MountainEnvironment>) {
	// Seed Cocoon's `__terminals` with any terminals already running.
	// `$acceptTerminalOpened` fires during `localPty:createProcess` but
	// Cocoon's gRPC channel is not yet open at that point (the workbench
	// restores terminals before Cocoon connects). Resend each open
	// terminal here so `vscode.window.terminals` is never empty for
	// extensions that read it synchronously in `activate()`.
	let ActiveTerminals = Environment.ApplicationState.Feature.Terminals.GetAll();

	if !ActiveTerminals.is_empty() {
		dev_log!(
			"terminal",
			"[CocoonManagement] Seeding {} open terminal(s) to Cocoon",
			ActiveTerminals.len()
		);

		for (Id, Terminal) in &ActiveTerminals {
			let Payload = serde_json::json!({
				"id": Id,
				"name": Terminal.Name,
				"pid": Terminal.OSProcessIdentifier,
			});

			let _ = crate::Vine::Client::SendNotification::Fn(
				SideCarId.to_string(),
				"$acceptTerminalOpened".to_string(),
				Payload,
			)
			.await;
		}
	}
}
