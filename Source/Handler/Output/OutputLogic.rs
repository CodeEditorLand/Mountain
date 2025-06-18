// @module OutputLogic
// @description Contains the core logic for managing output channels, including
// state management and emitting events to the Sky frontend for User Interface updates.

use Common::error::CommonError;
use log::{info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::ApplicationState::{ApplicationState::ApplicationState, DTO::OutputChannelStateDTO};

// Logic to register a new output channel.
pub async fn RegisterOutputChannelLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	name:String,
	language_identifier:Option<String>,
) -> Result<String, CommonError> {
	info!("[OutputLogic] Registering channel: '{}'", name);
	let channel_id = name.clone();
	let app_state = app_handle.state::<ApplicationState>();
	let mut channels_guard = app_state.OutputChannels.lock().unwrap();

	// Create the channel if it doesn't exist.
	channels_guard
		.entry(channel_id.clone())
		.or_insert_with(|| OutputChannelStateDTO::New(&name, language_identifier));
	drop(channels_guard);

	let event_payload = json!({ "Id": channel_id, "Name": name, "LanguageId": language_identifier });
	app_handle
		.emit("sky://output/create", event_payload)
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })?;
	Ok(name)
}

// Logic to append text to an output channel.
pub async fn AppendToOutputChannelLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	channel_identifier:String,
	value:String,
) -> Result<(), CommonError> {
	trace!("[OutputLogic] Appending to channel: '{}'", channel_identifier);
	let app_state = app_handle.state::<ApplicationState>();
	let mut channels_guard = app_state.OutputChannels.lock().unwrap();

	if let Some(channel_state) = channels_guard.get_mut(&channel_identifier) {
		channel_state.Buffer.push_str(&value);
		let event_payload = json!({ "Id": channel_identifier, "AppendedText": value });
		app_handle
			.emit("sky://output/append", event_payload)
			.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })?;
	} else {
		warn!("[OutputLogic] Channel '{}' not found for append.", channel_identifier);
	}
	Ok(())
}

// Logic to replace the entire content of an output channel.
pub async fn ReplaceOutputChannelContentLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	channel_identifier:String,
	value:String,
) -> Result<(), CommonError> {
	info!("[OutputLogic] Replacing content of channel: '{}'", channel_identifier);
	let app_state = app_handle.state::<ApplicationState>();
	let mut channels_guard = app_state.OutputChannels.lock().unwrap();

	if let Some(channel_state) = channels_guard.get_mut(&channel_identifier) {
		channel_state.Buffer = value.clone();
		let event_payload = json!({ "Id": channel_identifier, "Content": value });
		app_handle
			.emit("sky://output/replace", event_payload)
			.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })?;
	} else {
		warn!("[OutputLogic] Channel '{}' not found for replace.", channel_identifier);
	}
	Ok(())
}

// Logic to clear all content from an output channel.
pub async fn ClearOutputChannelLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	channel_identifier:String,
) -> Result<(), CommonError> {
	info!("[OutputLogic] Clearing channel: '{}'", channel_identifier);
	let app_state = app_handle.state::<ApplicationState>();
	let mut channels_guard = app_state.OutputChannels.lock().unwrap();

	if let Some(channel_state) = channels_guard.get_mut(&channel_identifier) {
		channel_state.Buffer.clear();
		app_handle
			.emit("sky://output/clear", json!({ "Id": channel_identifier }))
			.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })?;
	} else {
		warn!("[OutputLogic] Channel '{}' not found for clear.", channel_identifier);
	}
	Ok(())
}

// Logic to reveal an output channel in the User Interface.
pub async fn RevealOutputChannelLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	channel_identifier:String,
	preserve_focus:bool,
) -> Result<(), CommonError> {
	info!("[OutputLogic] Revealing channel: '{}'", channel_identifier);
	let app_state = app_handle.state::<ApplicationState>();
	let mut channels_guard = app_state.OutputChannels.lock().unwrap();

	if let Some(channel_state) = channels_guard.get_mut(&channel_identifier) {
		channel_state.Visible = true;
		let event_payload = json!({ "Id": channel_identifier, "PreserveFocus": preserve_focus });
		app_handle
			.emit("sky://output/reveal", event_payload)
			.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })?;
	} else {
		warn!("[OutputLogic] Channel '{}' not found for reveal.", channel_identifier);
	}
	Ok(())
}

// Logic to dispose of an output channel.
pub async fn DisposeOutputChannelLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	channel_identifier:String,
) -> Result<(), CommonError> {
	info!("[OutputLogic] Disposing channel: '{}'", channel_identifier);
	let app_state = app_handle.state::<ApplicationState>();
	app_state.OutputChannels.lock().unwrap().remove(&channel_identifier);
	app_handle
		.emit("sky://output/dispose", json!({ "Id": channel_identifier }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}
