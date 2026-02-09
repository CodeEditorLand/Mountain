//! # OutputProvider (Environment)
//!
//! Implements the
//! [`OutputChannelManager`](CommonLibrary::Output::OutputChannelManager) trait
//! for
//! [`MountainEnvironment`](crate::Environment::MountainEnvironment::MountainEnvironment).
//!
//! This provider manages multiple output channels (e.g., 'Extension Host',
//! 'JavaScript', 'Git'), handling channel lifecycle, content management, and UI
//! visibility. It maintains in-memory buffers with size limits and emits Tauri
//! events to the Sky frontend for UI updates.
//!
//! ## Implementation Strategy
//!
//! The trait implementation is split across multiple helper modules for
//! maintainability:
//! - [`ChannelLifecycle`]: `RegisterChannel`, `Dispose`
//! - [`ChannelContent`]: `Append`, `Replace`, `Clear`
//! - [`ChannelVisibility`]: `Reveal`, `Close`
//!
//! The single `impl OutputChannelManager for MountainEnvironment` block in this
//! file delegates to those helper functions. This satisfies Rust's orphan rules
//! while keeping code organized.

use CommonLibrary::Output::OutputChannelManager::OutputChannelManager;
use async_trait::async_trait;

// Private helper modules (not re-exported)
mod ChannelLifecycle;
mod ChannelContent;
mod ChannelVisibility;

#[async_trait]
impl OutputChannelManager for crate::Environment::MountainEnvironment::MountainEnvironment {
	async fn RegisterChannel(
		&self,
		name:String,
		language_identifier:Option<String>,
	) -> Result<String, CommonLibrary::Error::CommonError::CommonError> {
		ChannelLifecycle::register_channel(self, name, language_identifier).await
	}

	async fn Append(
		&self,
		channel_identifier:String,
		value:String,
	) -> Result<(), CommonLibrary::Error::CommonError::CommonError> {
		ChannelContent::append_to_channel(self, channel_identifier, value).await
	}

	async fn Replace(
		&self,
		channel_identifier:String,
		value:String,
	) -> Result<(), CommonLibrary::Error::CommonError::CommonError> {
		ChannelContent::replace_channel_content(self, channel_identifier, value).await
	}

	async fn Clear(&self, channel_identifier:String) -> Result<(), CommonLibrary::Error::CommonError::CommonError> {
		ChannelContent::clear_channel(self, channel_identifier).await
	}

	async fn Reveal(
		&self,
		channel_identifier:String,
		preserve_focus:bool,
	) -> Result<(), CommonLibrary::Error::CommonError::CommonError> {
		ChannelVisibility::reveal_channel(self, channel_identifier, preserve_focus).await
	}

	async fn Close(&self, channel_identifier:String) -> Result<(), CommonLibrary::Error::CommonError::CommonError> {
		ChannelVisibility::close_channel(self, channel_identifier).await
	}

	async fn Dispose(&self, channel_identifier:String) -> Result<(), CommonLibrary::Error::CommonError::CommonError> {
		ChannelLifecycle::dispose_channel(self, channel_identifier).await
	}
}
