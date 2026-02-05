//! # Output Channel Lifecycle Helpers
//!
//! Internal helper functions for output channel creation and disposal.
//! These are not public API - they are called by the main provider implementation.

use CommonLibrary::Error::CommonError::CommonError;
use log::{error, info};
use serde_json::json;
use tauri::Emitter;

use crate::ApplicationState::DTO::OutputChannelStateDTO::OutputChannelStateDTO;
use crate::Environment::Utility;

/// Registers a new output channel.
pub(super) async fn register_channel(
	env: &crate::Environment::MountainEnvironment::MountainEnvironment,
	name: String,
	language_identifier: Option<String>,
) -> Result<String, CommonError> {
	info!("[OutputProvider] Registering channel: '{}'", name);

	// Validate channel name
	if name.is_empty() {
		return Err(CommonError::InvalidArgument {
			ArgumentName: "Name".into(),
			Reason: "Channel name cannot be empty".into(),
		});
	}

	if name.len() > 256 {
		return Err(CommonError::InvalidArgument {
			ArgumentName: "Name".into(),
			Reason: "Channel name exceeds maximum length of 256 characters".into(),
		});
	}

	// Validate language identifier length if provided
	if let Some(ref lang_id) = language_identifier {
		if lang_id.len() > 64 {
			return Err(CommonError::InvalidArgument {
				ArgumentName: "LanguageIdentifier".into(),
				Reason: "Language identifier exceeds maximum length of 64 characters".into(),
			});
		}
	}

	let channel_identifier = name.clone();

	let mut channels_guard = env
		.ApplicationState
		.OutputChannels
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

	channels_guard.entry(channel_identifier.clone()).or_insert_with(|| {
		OutputChannelStateDTO::Create(&name, language_identifier.clone()).unwrap_or_else(|e| {
			error!("[OutputProvider] Failed to create output channel: {}", e);
			OutputChannelStateDTO::default()
		})
	});

	drop(channels_guard);

	let event_payload = json!({ "Id": channel_identifier, "Name": name, "LanguageId": language_identifier });

	env.ApplicationHandle
		.emit("sky://output/create", event_payload)
		.map_err(|Error| CommonError::UserInterfaceInteraction { Reason: Error.to_string() })?;

	Ok(channel_identifier)
}

/// Disposes of an output channel permanently.
pub(super) async fn dispose_channel(
	env: &crate::Environment::MountainEnvironment::MountainEnvironment,
	channel_identifier: String,
) -> Result<(), CommonError> {
	info!("[OutputProvider] Disposing channel: '{}'", channel_identifier);

	env.ApplicationState
		.OutputChannels
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
		.remove(&channel_identifier);

	env.ApplicationHandle
		.emit("sky://output/dispose", json!({ "Id": channel_identifier }))
		.map_err(|Error| CommonError::UserInterfaceInteraction { Reason: Error.to_string() })
}
