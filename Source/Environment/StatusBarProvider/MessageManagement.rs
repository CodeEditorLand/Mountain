//! # StatusBarProvider - Message Management
//!
//! Implementation of status bar temporary message handling for
//! [`MountainEnvironment`]

use CommonLibrary::{Error::CommonError::CommonError, IPC::SkyEvent::SkyEvent};
use serde_json::{Value, json};
use tauri::Emitter;

use super::super::MountainEnvironment::MountainEnvironment;
use crate::dev_log;

/// Message management operations implementation for MountainEnvironment
pub(super) async fn set_status_bar_message_impl(
	env:&MountainEnvironment,
	message_identifier:String,
	text:String,
) -> Result<(), CommonError> {
	dev_log!(
		"lifecycle",
		"[StatusBarProvider] Setting status message '{}': {}",
		message_identifier,
		text
	);

	env.ApplicationHandle
		.emit::<Value>(SkyEvent::StatusBarSetMessage.AsStr(), json!({ "id": message_identifier, "text": text }))
		.map_err(|error| CommonError::UserInterfaceInteraction { Reason:error.to_string() })
}

/// Disposes of a temporary status bar message.
pub(super) async fn dispose_status_bar_message_impl(
	env:&MountainEnvironment,
	message_identifier:String,
) -> Result<(), CommonError> {
	dev_log!(
		"lifecycle",
		"[StatusBarProvider] Disposing status message '{}'",
		message_identifier
	);

	env.ApplicationHandle
		.emit::<Value>(SkyEvent::StatusBarDisposeMessage.AsStr(), json!({ "id": message_identifier }))
		.map_err(|error| CommonError::UserInterfaceInteraction { Reason:error.to_string() })
}
