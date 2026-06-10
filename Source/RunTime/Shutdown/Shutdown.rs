//! Top-level shutdown orchestrator. Emits the `sky://lifecycle/willShutdown`
//! event so Wind/Sky can flush dirty editors, dispose sockets, and cancel
//! async tasks before the runtime tears down. Then calls
//! `ShutdownWithRecovery` and logs the outcome.

use tauri::Emitter;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

impl ApplicationRunTime {
	pub async fn Shutdown(&self) {
		dev_log!("lifecycle", "[ApplicationRunTime] Initiating graceful shutdown of services...");

		if let Err(Error) = self
			.Environment
			.ApplicationHandle
			.emit("sky://lifecycle/willShutdown", serde_json::json!({ "reason": "quit" }))
		{
			dev_log!(
				"lifecycle",
				"warn: [ApplicationRunTime] sky://lifecycle/willShutdown emit failed: {}",
				Error
			);
		}

		match self.ShutdownWithRecovery().await {
			Ok(()) => {
				dev_log!(
					"lifecycle",
					"[ApplicationRunTime] Service shutdown tasks completed successfully."
				)
			},

			Err(Error) => {
				dev_log!(
					"lifecycle",
					"error: [ApplicationRunTime] Service shutdown completed with errors: {}",
					Error
				)
			},
		}
	}
}
