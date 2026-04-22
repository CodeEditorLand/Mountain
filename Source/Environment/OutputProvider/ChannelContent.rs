//! # Output Channel Content Helpers
//!
//! Internal helper functions for output channel content manipulation.
//! These are not public API - they are called by the main provider
//! implementation.

use CommonLibrary::{Error::CommonError::CommonError, IPC::SkyEvent::SkyEvent};
use serde_json::json;
use tauri::Emitter;

use crate::{Environment::Utility, dev_log};

/// Appends text to an output channel.
/// Includes buffer size validation to prevent memory exhaustion.
pub(super) async fn append_to_channel(
	env:&crate::Environment::MountainEnvironment::MountainEnvironment,
	channel_identifier:String,
	value:String,
) -> Result<(), CommonError> {
	dev_log!("output", "[OutputProvider] Appending to channel: '{}'", channel_identifier);

	// Validate input size to prevent memory exhaustion
	if value.len() > 1_048_576 {
		// 1MB limit per append
		return Err(CommonError::InvalidArgument {
			ArgumentName:"Value".into(),
			Reason:"Append value exceeds maximum size of 1MB".into(),
		});
	}

	let mut channels_guard = env
		.ApplicationState
		.Feature
		.OutputChannels
		.OutputChannels
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

	if let Some(channel_state) = channels_guard.get_mut(&channel_identifier) {
		// Enforce total buffer size limit of 10MB per channel to prevent
		// unbounded memory growth from excessive output accumulation.
		const MAX_BUFFER_SIZE:usize = 10 * 1_048_576;
		if channel_state.Buffer.len() + value.len() > MAX_BUFFER_SIZE {
			// Trim from beginning to make room for new content.
			// Keep 1MB headroom to avoid frequent reallocation.
			let trim_size:usize = value.len() + 1_048_576;
			if channel_state.Buffer.len() > trim_size {
				let _ = channel_state.Buffer.drain(..trim_size);
			}
		}

		channel_state.Buffer.push_str(&value);

		let event_payload = json!({ "Id": channel_identifier, "AppendedText": value });

		env.ApplicationHandle
			.emit(SkyEvent::OutputAppend.AsStr(), event_payload)
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
	} else {
		dev_log!(
			"output",
			"warn: [OutputProvider] Channel '{}' not found for append.",
			channel_identifier
		);
	}

	Ok(())
}

/// Replaces the entire content of an output channel.
pub(super) async fn replace_channel_content(
	env:&crate::Environment::MountainEnvironment::MountainEnvironment,
	channel_identifier:String,
	value:String,
) -> Result<(), CommonError> {
	dev_log!(
		"output",
		"[OutputProvider] Replacing content of channel: '{}'",
		channel_identifier
	);

	let mut channels_guard = env
		.ApplicationState
		.Feature
		.OutputChannels
		.OutputChannels
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

	if let Some(channel_state) = channels_guard.get_mut(&channel_identifier) {
		channel_state.Buffer = value.clone();

		let event_payload = json!({ "Id": channel_identifier, "Content": value });

		env.ApplicationHandle
			.emit(SkyEvent::OutputReplace.AsStr(), event_payload)
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
	} else {
		dev_log!(
			"output",
			"warn: [OutputProvider] Channel '{}' not found for replace.",
			channel_identifier
		);
	}

	Ok(())
}

/// Clears all content from an output channel.
pub(super) async fn clear_channel(
	env:&crate::Environment::MountainEnvironment::MountainEnvironment,
	channel_identifier:String,
) -> Result<(), CommonError> {
	dev_log!("output", "[OutputProvider] Clearing channel: '{}'", channel_identifier);

	let mut channels_guard = env
		.ApplicationState
		.Feature
		.OutputChannels
		.OutputChannels
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

	if let Some(channel_state) = channels_guard.get_mut(&channel_identifier) {
		channel_state.Buffer.clear();

		env.ApplicationHandle
			.emit(SkyEvent::OutputClear.AsStr(), json!({ "Id": channel_identifier }))
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
	} else {
		dev_log!(
			"output",
			"warn: [OutputProvider] Channel '{}' not found for clear.",
			channel_identifier
		);
	}

	Ok(())
}
