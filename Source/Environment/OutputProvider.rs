// File: Mountain/Source/Environment/OutputProvider.rs
// Role: Implements the `OutputChannelManager` trait for the
// `MountainEnvironment`. Responsibilities:
//   - Manage multiple output channels (e.g., 'Extension Host', 'JavaScript', 'Git')
//   - Handle channel creation, modification, and disposal.
//   - Emit events to the Sky frontend for UI updates.
//   - Manage output formatting and encoding.
//   - Handle channel scoping and visibility state.
//   - Maintain output buffer in memory for persistence.
//
// TODOs:
//   - Implement output formatting (syntax highlighting, line numbers)
//   - Add output encoding support (UTF-8, UTF-16, ASCII)
//   - Implement output buffering with size limits
//   - Add output channel persistence to disk
//   - Implement output channel export functionality
//   - Add output search and filtering
//   - Support output channel grouping
//   - Implement output timestamping
//   - Add output channel priority (relevance ordering)
//   - Support output channel logging levels
//   - Implement output deduplication
//   - Add output channel statistics (line count, char count)
//   - Implement output scroll-to-bottom behavior
//   - Support output word wrap configuration
//
// Inspired by VSCode's output service which:
// - Separates output channels by identifier
// - Supports output visibility management
// - Handles large output buffers efficiently
// - Provides output channel language-specific formatting
// - Manages output channel lifecycle and disposal

//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//!
//! # OutputProvider Implementation
//!
//! Implements the `OutputChannelManager` trait for the `MountainEnvironment`.
//! This provider contains the core logic for managing output channels,
//! including state management and emitting events to the `Sky` frontend for UI
//! updates.
//!
//! ## Channel Management
//!
//! Each output channel maintains:
//! - A unique identifier (channel name)
//! - A display name (shown in the UI)
//! - An optional language identifier for syntax highlighting
//! - An in-memory buffer of output content
//! - A visibility state flag
//!
//! ## Channel Lifecycle
//!
//! 1. **Register**: Create a new channel with the specified name and language
//! 2. **Append**: Add text to the channel's buffer (emit append event)
//! 3. **Replace**: Replace the entire channel content (emit replace event)
//! 4. **Clear**: Empty the channel buffer (emit clear event)
//! 5. **Reveal**: Show the channel in the UI with optional focus preservation
//! 6. **Close**: Hide the channel view (channel remains registered)
//! 7. **Dispose**: Permanently remove the channel from memory and UI
//!
//! ## Output Scoping
//!
//! Output channels are scoped by extension or feature:
//! - Built-in channels: 'Extension Host', 'Tasks', 'Debug'
//! - Extension channels: Created by extensions with their own naming scheme
//! - Channels can be shown/hide individually or batch managed

#![allow(non_snake_case, non_camel_case_types)]

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
	/// Includes validation for channel name and language identifier.
	async fn RegisterChannel(&self, Name:String, LanguageIdentifier:Option<String>) -> Result<String, CommonError> {
		info!("[OutputProvider] Registering channel: '{}'", Name);

		// Validate channel name
		if Name.is_empty() {
			return Err(CommonError::InvalidArgument {
				ArgumentName: "Name".into(),
				Reason: "Channel name cannot be empty".into(),
			});
		}

		if Name.len() > 256 {
			return Err(CommonError::InvalidArgument {
				ArgumentName: "Name".into(),
				Reason: "Channel name exceeds maximum length of 256 characters".into(),
			});
		}

		// Validate language identifier length if provided
		if let Some(ref lang_id) = LanguageIdentifier {
			if lang_id.len() > 64 {
				return Err(CommonError::InvalidArgument {
					ArgumentName: "LanguageIdentifier".into(),
					Reason: "Language identifier exceeds maximum length of 64 characters".into(),
				});
			}
		}

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
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;

		Ok(ChannelIdentifier)
	}

	/// Appends text to an output channel.
	/// Includes buffer size validation to prevent memory exhaustion.
	async fn Append(&self, ChannelIdentifier:String, Value:String) -> Result<(), CommonError> {
		trace!("[OutputProvider] Appending to channel: '{}'", ChannelIdentifier);

		// Validate input size to prevent memory exhaustion
		if Value.len() > 1_048_576 { // 1MB limit per append
			return Err(CommonError::InvalidArgument {
				ArgumentName: "Value".into(),
				Reason: "Append value exceeds maximum size of 1MB".into(),
			});
		}

		let mut ChannelsGuard = self
			.ApplicationState
			.OutputChannels
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		if let Some(ChannelState) = ChannelsGuard.get_mut(&ChannelIdentifier) {
			// Check buffer size before appending
			const MAX_BUFFER_SIZE: usize = 10 * 1_048_576; // 10MB total buffer limit
			if ChannelState.Buffer.len() + Value.len() > MAX_BUFFER_SIZE {
				// Trim from beginning to make room
				const TRIM_SIZE: usize = Value.len() + 1_048_576; // Keep 1MB headroom
				if ChannelState.Buffer.len() > TRIM_SIZE {
					let _ = ChannelState.Buffer.drain(..TRIM_SIZE);
				}
			}

			ChannelState.Buffer.push_str(&Value);

			let EventPayload = json!({ "Id": ChannelIdentifier, "AppendedText": Value });

			self.ApplicationHandle
				.emit("sky://output/append", EventPayload)
				.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
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
				.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
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
				.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
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
				.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })?;
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
			.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:Error.to_string() })
	}
}
