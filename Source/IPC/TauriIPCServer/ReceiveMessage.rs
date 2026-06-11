//! Validates permissions, routes an incoming Wind IPC Message to the
//! managed `TauriIPCServer` state, and records performance metrics.
//! Body of the `mountain_ipc_receive_message` command.

use tauri::Manager;

use super::{SecurityEvent, SecurityEventType, TauriIPCMessage, TauriIPCServer};
use crate::dev_log;

pub(crate) async fn Fn(app_handle:tauri::AppHandle, Message:TauriIPCMessage) -> Result<(), String> {
	dev_log!(
		"ipc",
		"[TauriIPCServer] Received IPC Message from Wind on channel: {}",
		Message.channel
	);

	// Get the IPC server instance from application state
	if let Some(ipc_server) = app_handle.try_state::<TauriIPCServer>() {
		// Advanced security: Validate permissions before processing
		if let Err(e) = ipc_server.validate_message_permissions(&Message).await {
			dev_log!(
				"ipc",
				"error: [TauriIPCServer] Permission validation failed for channel {}: {}",
				Message.channel,
				e
			);

			// Log security event
			ipc_server
				.log_security_event(SecurityEvent {
					event_type:SecurityEventType::PermissionDenied,
					user_id:Message.sender.clone().unwrap_or("unknown".to_string()),
					operation:Message.channel.clone(),
					timestamp:std::time::SystemTime::now(),
					details:Some(format!("Permission denied: {}", e)),
				})
				.await;

			return Err(format!("Permission denied: {}", e));
		}

		// Advanced monitoring: Track Message processing time
		let start_time = std::time::Instant::now();

		let result = ipc_server.IncomingMessage(Message.clone()).await;

		let duration = start_time.elapsed();

		// Record performance metrics
		ipc_server
			.record_performance_metrics(Message.channel, duration, result.is_ok())
			.await;

		result
	} else {
		Err("IPC Server not found in application state".to_string())
	}
}
