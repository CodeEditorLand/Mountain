//! # OutputProvider (Environment)
//!
//! RESPONSIBILITIES:
//! - Implements [`OutputChannelManager`](CommonLibrary::Output::OutputChannelManager) for [`MountainEnvironment`]
//! - Manages multiple output channels (e.g., 'Extension Host', 'JavaScript', 'Git')
//! - Handles channel lifecycle: creation, appending, replacement, clearing, disposal
//! - Maintains in-memory buffers with size limits and automatic trimming
//! - Emits Tauri events to Sky frontend for UI updates
//! - Controls channel visibility and reveal behavior
//!
//! ARCHITECTURAL ROLE:
//! - Environment provider for output channel management
//! - Channels stored in [`ApplicationState.OutputChannels`](crate::ApplicationState::ApplicationState)
//! - Uses Tauri event system (`tauri::Emitter`) for UI communication
//! - Integrates with [`Utility`](crate::Utility) for state lock error handling
//! - Buffer management with 10MB limit per channel and 1MB trim threshold
//!
//! ERROR HANDLING:
//! - Uses [`CommonError`](CommonLibrary::Error::CommonError) for all operations
//! - Validates channel name length (1-256 chars) and language identifier (max 64 chars)
//! - Rejects append values exceeding 1MB per operation
//! - State lock errors mapped via [`Utility::MapApplicationStateLockErrorToCommonError`](crate::Utility)
//! - Warns on operations for non-existent channels (Append, Replace, Clear, Reveal)
//! - Close operation is stubbed with warning
//!
//! PERFORMANCE:
//! - In-memory buffer with O(1) append and O(n) replace operations
//! - Automatic buffer trimming when approaching 10MB limit (drains from front)
//! - Lock on shared state should be minimized; drops guard promptly after mutation
//! - TODO: Consider streaming large outputs directly to UI without buffering
//!
//! VS CODE REFERENCE:
//! - `vs/workbench/contrib/output/common/output.ts` - output channel service
//! - `vs/workbench/services/output/common/outputService.ts` - output service main logic
//! - `vs/workbench/contrib/output/browser/outputPanel.ts` - output view UI
//!
//! TODO:
//! - Implement output formatting (syntax highlighting, line numbers)
//! - Add output encoding support (UTF-8, UTF-16, ASCII)
//! - Implement output buffering with configurable size limits
//! - Add output channel persistence to disk for recovery
//! - Implement output channel export functionality (save to file)
//! - Add output search and filtering capabilities
//! - Support output channel grouping and categorization
//! - Implement output timestamping for each line
//! - Add output channel priority (relevance ordering)
//! - Support output channel logging levels (debug, info, warn, error)
//! - Implement output deduplication to reduce noise
//! - Add output channel statistics (line count, char count, age)
//! - Implement output scroll-to-bottom behavior configuration
//! - Support output word wrap configuration
//! - Implement Close operation fully (hide channel and cleanup)
//!
//! MODULE CONTENTS:
//! - [`OutputChannelManager`](CommonLibrary::Output::OutputChannelManager) implementation:
//!   - [`RegisterChannel`](Self::RegisterChannel) - create new channel with validation
//!   - [`Append`](Self::Append) - add text with buffer size enforcement (1MB per call)
//!   - [`Replace`](Self::Replace) - replace entire channel buffer
//!   - [`Clear`](Self::Clear) - empty channel buffer
//!   - [`Reveal`](Self::Reveal) - show channel in UI with focus option
//!   - [`Close`](Self::Close) - hide channel (stub)
//!   - [`Dispose`](Self::Dispose) - permanently remove channel from state
//! - Data type: [`OutputChannelStateDTO`](crate::ApplicationState::DTO::OutputChannelStateDTO)

use CommonLibrary::{Error::CommonError::CommonError, Output::OutputChannelManager::OutputChannelManager};
use async_trait::async_trait;
use log::{error, info, trace, warn};
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
				ArgumentName:"Name".into(),
				Reason:"Channel name cannot be empty".into(),
			});
		}

		if Name.len() > 256 {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"Name".into(),
				Reason:"Channel name exceeds maximum length of 256 characters".into(),
			});
		}

		// Validate language identifier length if provided
		if let Some(ref lang_id) = LanguageIdentifier {
			if lang_id.len() > 64 {
				return Err(CommonError::InvalidArgument {
					ArgumentName:"LanguageIdentifier".into(),
					Reason:"Language identifier exceeds maximum length of 64 characters".into(),
				});
			}
		}

		let ChannelIdentifier = Name.clone();

		let mut ChannelsGuard = self
			.ApplicationState
			.OutputChannels
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		ChannelsGuard.entry(ChannelIdentifier.clone()).or_insert_with(|| {
			OutputChannelStateDTO::Create(&Name, LanguageIdentifier.clone()).unwrap_or_else(|e| {
				error!("[OutputProvider] Failed to create output channel: {}", e);
				OutputChannelStateDTO::default()
			})
		});

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
		if Value.len() > 1_048_576 {
			// 1MB limit per append
			return Err(CommonError::InvalidArgument {
				ArgumentName:"Value".into(),
				Reason:"Append value exceeds maximum size of 1MB".into(),
			});
		}

		let mut ChannelsGuard = self
			.ApplicationState
			.OutputChannels
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		if let Some(ChannelState) = ChannelsGuard.get_mut(&ChannelIdentifier) {
			// Check buffer size before appending
			const MAX_BUFFER_SIZE:usize = 10 * 1_048_576; // 10MB total buffer limit
			if ChannelState.Buffer.len() + Value.len() > MAX_BUFFER_SIZE {
				// Trim from beginning to make room
				let TRIM_SIZE:usize = Value.len() + 1_048_576; // Keep 1MB headroom
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
