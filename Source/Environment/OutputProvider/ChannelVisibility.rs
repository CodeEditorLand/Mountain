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
		.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

	if let Some(channel_state) = channels_guard.get_mut(&channel_identifier) {
		channel_state.IsVisible = true;

		let event_payload = json!({ "channel": channel_identifier, "preserveFocus": preserve_focus });

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

/// Closes the view of an output channel in the UI. Hides the channel
/// (mutates `IsVisible` in `ApplicationState`) and emits a Sky event
/// so the renderer can collapse the panel; the channel itself stays
/// in state with its buffered lines so a later `reveal` can re-open
/// it without losing content. To remove the channel entirely, use
/// `dispose_channel` instead.
pub(super) async fn close_channel(
	env:&crate::Environment::MountainEnvironment::MountainEnvironment,
	channel_identifier:String,
) -> Result<(), CommonError> {
	dev_log!("output", "[OutputProvider] Closing channel: '{}'", channel_identifier);

	let mut channels_guard = env
		.ApplicationState
		.Feature
		.OutputChannels
		.OutputChannels
		.lock()
		.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

	if let Some(channel_state) = channels_guard.get_mut(&channel_identifier) {
		channel_state.IsVisible = false;
		// Re-use OutputReveal with `PreserveFocus: false` to push the
		// updated visibility state - SkyEvent doesn't yet have a
		// dedicated Hide variant; the renderer's reveal handler is
		// idempotent and reads the latest IsVisible from state.
		let event_payload = json!({ "channel": channel_identifier, "preserveFocus": true, "visible": false });
		env.ApplicationHandle
			.emit(SkyEvent::OutputReveal.AsStr(), event_payload)
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
	} else {
		dev_log!(
			"output",
			"warn: [OutputProvider] Channel '{}' not found for close.",
			channel_identifier
		);
	}

	Ok(())
}
