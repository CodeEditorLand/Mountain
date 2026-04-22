//! # Output Channel Visibility Helpers
//!
//! Internal helper functions for output channel UI visibility operations.
//! These are not public API - they are called by the main provider
//! implementation.

use CommonLibrary::{Error::CommonError::CommonError, IPC::SkyEvent::SkyEvent};
use serde_json::json;
use tauri::Emitter;

use crate::{Environment::Utility, dev_log};

/// Reveals an output channel in the UI.
pub(super) async fn reveal_channel(
	env:&crate::Environment::MountainEnvironment::MountainEnvironment,
	channel_identifier:String,
	preserve_focus:bool,
) -> Result<(), CommonError> {
	dev_log!("output", "[OutputProvider] Revealing channel: '{}'", channel_identifier);

	let mut channels_guard = env
		.ApplicationState
		.Feature
		.OutputChannels
		.OutputChannels
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

	if let Some(channel_state) = channels_guard.get_mut(&channel_identifier) {
		channel_state.IsVisible = true;

		let event_payload = json!({ "Id": channel_identifier, "PreserveFocus": preserve_focus });

		env.ApplicationHandle
			.emit(SkyEvent::OutputReveal.AsStr(), event_payload)
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
	} else {
		dev_log!(
			"output",
			"warn: [OutputProvider] Channel '{}' not found for reveal.",
			channel_identifier
		);
	}

	Ok(())
}

/// Closes the view of an output channel in the UI.
pub(super) async fn close_channel(
	_env:&crate::Environment::MountainEnvironment::MountainEnvironment,
	_channel_identifier:String,
) -> Result<(), CommonError> {
	dev_log!("output", "warn: [OutputProvider] Close is not fully implemented.");

	Ok(())
}
