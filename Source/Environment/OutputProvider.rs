
// Implements the `OutputChannelManager` trait for the `MountainEnvironment`.
// This file connects abstract output channel effects to the concrete logic
// in the application's output handlers.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{Environment::Requires, Errors::CommonError, OutputEffect::OutputChannelManager};
use async_trait::async_trait;
use log::{info, trace};

use crate::{Environment::MountainEnvironment, Handlers};

#[async_trait]
impl OutputChannelManager for MountainEnvironment {
	/// Registers a new output channel.
	async fn RegisterChannel(&self, Name:String, LanguageIdentifier:Option<String>) -> Result<String, CommonError> {
		info!(
			"[Environment OutputProvider] RegisterChannel: Name='{}', LanguageIdentifier='{:?}'",
			Name, LanguageIdentifier
		);
		Handlers::Output::HandleRegisterOutputChannelEffectLogic(self.AppHandle.clone(), Name, LanguageIdentifier).await
	}

	/// Appends a string to an existing channel.
	async fn Append(&self, ChannelIdentifier:String, Value:String) -> Result<(), CommonError> {
		trace!(
			"[Environment OutputProvider] Append: ChannelIdentifier='{}', ValueLength={}",
			ChannelIdentifier,
			Value.len()
		);
		Handlers::Output::HandleAppendToOutputChannelEffectLogic(self.AppHandle.clone(), ChannelIdentifier, Value).await
	}

	/// Replaces the entire content of a channel with a new string.
	async fn Replace(&self, ChannelIdentifier:String, Value:String) -> Result<(), CommonError> {
		info!(
			"[Environment OutputProvider] Replace: ChannelIdentifier='{}', NewValueLength={}",
			ChannelIdentifier,
			Value.len()
		);
		Handlers::Output::HandleReplaceOutputChannelContentEffectLogic(self.AppHandle.clone(), ChannelIdentifier, Value)
			.await
	}

	/// Clears all content from a channel.
	async fn Clear(&self, ChannelIdentifier:String) -> Result<(), CommonError> {
		info!("[Environment OutputProvider] Clear: ChannelIdentifier='{}'", ChannelIdentifier);
		Handlers::Output::HandleClearOutputChannelEffectLogic(self.AppHandle.clone(), ChannelIdentifier).await
	}

	/// Makes a channel visible in the UI.
	async fn Reveal(&self, ChannelIdentifier:String, PreserveFocus:bool) -> Result<(), CommonError> {
		info!(
			"[Environment OutputProvider] Reveal: ChannelIdentifier='{}', PreserveFocus={}",
			ChannelIdentifier, PreserveFocus
		);
		Handlers::Output::HandleRevealOutputChannelEffectLogic(self.AppHandle.clone(), ChannelIdentifier, PreserveFocus)
			.await
	}

	/// Hides a channel's view in the UI.
	async fn Close(&self, ChannelIdentifier:String) -> Result<(), CommonError> {
		info!("[Environment OutputProvider] Close: ChannelIdentifier='{}'", ChannelIdentifier);
		Handlers::Output::HandleCloseOutputChannelViewEffectLogic(self.AppHandle.clone(), ChannelIdentifier).await
	}

	/// Completely removes and disposes of a channel.
	async fn Dispose(&self, ChannelIdentifier:String) -> Result<(), CommonError> {
		info!(
			"[Environment OutputProvider] Dispose: ChannelIdentifier='{}'",
			ChannelIdentifier
		);
		Handlers::Output::HandleDisposeOutputChannelEffectLogic(self.AppHandle.clone(), ChannelIdentifier).await
	}
}

impl Requires<Arc<dyn OutputChannelManager + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn OutputChannelManager + Send + Sync> { Arc::new(self.clone()) }
}
