use Common::error::CommonError;
use log::{info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager, Runtime};

/// @module OutputLogic
/// @description Contains the core logic for managing output channels, including
/// state management and emitting events to the Sky frontend for UI updates.
use crate::AppState::{AppState::AppState, Dto::OutputChannelStateDto};

/// Logic to register a new output channel.
pub async fn RegisterOutputChannelLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	Name:String,
	LanguageIdentifier:Option<String>,
) -> Result<String, CommonError> {
	info!("[OutputLogic] Registering channel: '{}'", Name);
	let ChannelId = Name.clone();
	let AppStateInstance = AppHandle.state::<AppState>();
	let mut ChannelsGuard = AppStateInstance.OutputChannels.lock().unwrap();

	// Create the channel if it doesn't exist.
	ChannelsGuard
		.entry(ChannelId.clone())
		.or_insert_with(|| OutputChannelStateDto::New(&Name, LanguageIdentifier));
	drop(ChannelsGuard);

	let EventPayload = json!({ "Id": ChannelId, "Name": Name, "LanguageId": LanguageIdentifier });
	AppHandle
		.emit("sky://output/create", EventPayload)
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })?;
	Ok(ChannelId)
}

/// Logic to append text to an output channel.
pub async fn AppendToOutputChannelLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	ChannelIdentifier:String,
	Value:String,
) -> Result<(), CommonError> {
	trace!("[OutputLogic] Appending to channel: '{}'", ChannelIdentifier);
	let AppStateInstance = AppHandle.state::<AppState>();
	let mut ChannelsGuard = AppStateInstance.OutputChannels.lock().unwrap();

	if let Some(ChannelState) = ChannelsGuard.get_mut(&ChannelIdentifier) {
		ChannelState.Buffer.push_str(&Value);
		let EventPayload = json!({ "Id": ChannelIdentifier, "AppendedText": Value });
		AppHandle
			.emit("sky://output/append", EventPayload)
			.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })?;
	} else {
		warn!("[OutputLogic] Channel '{}' not found for append.", ChannelIdentifier);
	}
	Ok(())
}

/// Logic to replace the entire content of an output channel.
pub async fn ReplaceOutputChannelContentLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	ChannelIdentifier:String,
	Value:String,
) -> Result<(), CommonError> {
	info!("[OutputLogic] Replacing content of channel: '{}'", ChannelIdentifier);
	let AppStateInstance = AppHandle.state::<AppState>();
	let mut ChannelsGuard = AppStateInstance.OutputChannels.lock().unwrap();

	if let Some(ChannelState) = ChannelsGuard.get_mut(&ChannelIdentifier) {
		ChannelState.Buffer = Value.clone();
		let EventPayload = json!({ "Id": ChannelIdentifier, "Content": Value });
		AppHandle
			.emit("sky://output/replace", EventPayload)
			.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })?;
	} else {
		warn!("[OutputLogic] Channel '{}' not found for replace.", ChannelIdentifier);
	}
	Ok(())
}

/// Logic to clear all content from an output channel.
pub async fn ClearOutputChannelLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	ChannelIdentifier:String,
) -> Result<(), CommonError> {
	info!("[OutputLogic] Clearing channel: '{}'", ChannelIdentifier);
	let AppStateInstance = AppHandle.state::<AppState>();
	let mut ChannelsGuard = AppStateInstance.OutputChannels.lock().unwrap();

	if let Some(ChannelState) = ChannelsGuard.get_mut(&ChannelIdentifier) {
		ChannelState.Buffer.clear();
		AppHandle
			.emit("sky://output/clear", json!({ "Id": ChannelIdentifier }))
			.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })?;
	} else {
		warn!("[OutputLogic] Channel '{}' not found for clear.", ChannelIdentifier);
	}
	Ok(())
}

/// Logic to reveal an output channel in the UI.
pub async fn RevealOutputChannelLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	ChannelIdentifier:String,
	PreserveFocus:bool,
) -> Result<(), CommonError> {
	info!("[OutputLogic] Revealing channel: '{}'", ChannelIdentifier);
	let AppStateInstance = AppHandle.state::<AppState>();
	let mut ChannelsGuard = AppStateInstance.OutputChannels.lock().unwrap();

	if let Some(ChannelState) = ChannelsGuard.get_mut(&ChannelIdentifier) {
		ChannelState.Visible = true;
		let EventPayload = json!({ "Id": ChannelIdentifier, "PreserveFocus": PreserveFocus });
		AppHandle
			.emit("sky://output/reveal", EventPayload)
			.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })?;
	}
	Ok(())
}

/// Logic to dispose of an output channel.
pub async fn DisposeOutputChannelLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	ChannelIdentifier:String,
) -> Result<(), CommonError> {
	info!("[OutputLogic] Disposing channel: '{}'", ChannelIdentifier);
	let AppStateInstance = AppHandle.state::<AppState>();
	AppStateInstance.OutputChannels.lock().unwrap().remove(&ChannelIdentifier);
	AppHandle
		.emit("sky://output/dispose", json!({ "Id": ChannelIdentifier }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}
