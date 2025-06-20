//! # OutputProvider Implementation
//!
//! Implements the `OutputChannelManager` trait for the `MountainEnvironment`.
//! This provider contains the core logic for managing output channels,
//! including state management and emitting events to the `Sky` frontend for UI
//! updates.

use Common::{Error::CommonError::CommonError, Output::OutputChannelManager::OutputChannelManager};
use async_trait::async_trait;
use log::{info, trace, warn};
use serde_json::json;
use tauri::Emitter;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::ApplicationState::DTO::OutputChannelStateDTO::OutputChannelStateDTO;

#[async_trait]
impl OutputChannelManager for MountainEnvironment {
	/// Registers a new output channel.
	async fn RegisterChannel(&self, Name:String, LanguageIdentifier:Option<String>) -> Result<String, CommonError> {
		info!("[OutputProvider] Registering channel: '{}'", Name);
		let ChannelIdentifier = Name.clone();
		let mut ChannelsGuard = self
			.ApplicationState
			.OutputChannels
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		ChannelsGuard
			.entry(ChannelIdentifier.clone())
			.or_insert_with(|| OutputChannelStateDTO::Create(&Name, LanguageIdentifier.clone()));
		drop(ChannelsGuard);

		let EventPayload = json!({ "Id": ChannelIdentifier, "Name": Name, "LanguageId": LanguageIdentifier });
		self.ApplicationHandle
			.emit("sky://output/create", EventPayload)
			.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })?;
		Ok(ChannelIdentifier)
	}

	/// Appends text to an output channel.
	async fn Append(&self, ChannelIdentifier:String, Value:String) -> Result<(), CommonError> {
		trace!("[OutputProvider] Appending to channel: '{}'", ChannelIdentifier);
		let mut ChannelsGuard = self
			.ApplicationState
			.OutputChannels
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		if let Some(ChannelState) = ChannelsGuard.get_mut(&ChannelIdentifier) {
			ChannelState.Buffer.push_str(&Value);
			let EventPayload = json!({ "Id": ChannelIdentifier, "AppendedText": Value });
			self.ApplicationHandle
				.emit("sky://output/append", EventPayload)
				.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })?;
		} else {
			warn!("[OutputProvider] Channel '{}' not found for append.", ChannelIdentifier);
		}
		Ok(())
	}

	/// Replaces the entire content of an output channel.
	async fn Replace(&self, ChannelIdentifier:String, Value:String) -> Result<(), CommonError> {
		info!("[OutputProvider] Replacing content of channel: '{}'", ChannelIdentifier);
		let mut ChannelsGuard = self
			.ApplicationState
			.OutputChannels
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		if let Some(ChannelState) = ChannelsGuard.get_mut(&ChannelIdentifier) {
			ChannelState.Buffer = Value.clone();
			let EventPayload = json!({ "Id": ChannelIdentifier, "Content": Value });
			self.ApplicationHandle
				.emit("sky://output/replace", EventPayload)
				.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })?;
		} else {
			warn!("[OutputProvider] Channel '{}' not found for replace.", ChannelIdentifier);
		}
		Ok(())
	}

	/// Clears all content from an output channel.
	async fn Clear(&self, ChannelIdentifier:String) -> Result<(), CommonError> {
		info!("[OutputProvider] Clearing channel: '{}'", ChannelIdentifier);
		let mut ChannelsGuard = self
			.ApplicationState
			.OutputChannels
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		if let Some(ChannelState) = ChannelsGuard.get_mut(&ChannelIdentifier) {
			ChannelState.Buffer.clear();
			self.ApplicationHandle
				.emit("sky://output/clear", json!({ "Id": ChannelIdentifier }))
				.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })?;
		} else {
			warn!("[OutputProvider] Channel '{}' not found for clear.", ChannelIdentifier);
		}
		Ok(())
	}

	/// Reveals an output channel in the UI.
	async fn Reveal(&self, ChannelIdentifier:String, PreserveFocus:bool) -> Result<(), CommonError> {
		info!("[OutputProvider] Revealing channel: '{}'", ChannelIdentifier);
		let mut ChannelsGuard = self
			.ApplicationState
			.OutputChannels
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		if let Some(ChannelState) = ChannelsGuard.get_mut(&ChannelIdentifier) {
			ChannelState.IsVisible = true;
			let EventPayload = json!({ "Id": ChannelIdentifier, "PreserveFocus": PreserveFocus });
			self.ApplicationHandle
				.emit("sky://output/reveal", EventPayload)
				.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })?;
		} else {
			warn!("[OutputProvider] Channel '{}' not found for reveal.", ChannelIdentifier);
		}
		Ok(())
	}

	/// Closes the view of an output channel in the UI.
	async fn Close(&self, _ChannelIdentifier:String) -> Result<(), CommonError> {
		warn!("[OutputProvider] Close is not fully implemented.");
		Ok(())
	}

	/// Disposes of an output channel permanently.
	async fn Dispose(&self, ChannelIdentifier:String) -> Result<(), CommonError> {
		info!("[OutputProvider] Disposing channel: '{}'", ChannelIdentifier);
		self.ApplicationState
			.OutputChannels
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.remove(&ChannelIdentifier);
		self.ApplicationHandle
			.emit("sky://output/dispose", json!({ "Id": ChannelIdentifier }))
			.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })
	}
}
