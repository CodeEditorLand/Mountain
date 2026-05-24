//! `Shutdown::Shutdown`

use tauri::Emitter;

use super::Struct;
use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub fn Fn(This:&Struct) {
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

	match This.ShutdownWithRecovery().await {
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
