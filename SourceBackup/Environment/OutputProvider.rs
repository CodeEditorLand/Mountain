// @module OutputProvider (Environment)
// @description Implements the `OutputProvider` trait for `MountainEnvironment`.

use std::sync::Arc;

use async_trait::async_trait;
use Common::{Environment::Requires, error::CommonError, output::OutputProvider};

use super::MountainEnvironment;
use crate::Handler::output as OutputHandler;

#[async_trait]
impl OutputProvider for MountainEnvironment {
	async fn RegisterChannel(&self, name:String, language_identifier:Option<String>) -> Result<String, CommonError> {
		OutputHandler::RegisterOutputChannelLogic(&self.ApplicationHandle, name, language_identifier).await
	}

	async fn Append(&self, channel_identifier:String, value:String) -> Result<(), CommonError> {
		OutputHandler::AppendToOutputChannelLogic(&self.ApplicationHandle, channel_identifier, value).await
	}

	async fn Replace(&self, channel_identifier:String, value:String) -> Result<(), CommonError> {
		OutputHandler::ReplaceOutputChannelContentLogic(&self.ApplicationHandle, channel_identifier, value).await
	}

	async fn Clear(&self, channel_identifier:String) -> Result<(), CommonError> {
		OutputHandler::ClearOutputChannelLogic(&self.ApplicationHandle, channel_identifier).await
	}

	async fn Reveal(&self, channel_identifier:String, preserve_focus:bool) -> Result<(), CommonError> {
		OutputHandler::RevealOutputChannelLogic(&self.ApplicationHandle, channel_identifier, preserve_focus).await
	}

	async fn Dispose(&self, channel_identifier:String) -> Result<(), CommonError> {
		OutputHandler::DisposeOutputChannelLogic(&self.ApplicationHandle, channel_identifier).await
	}
}

impl Requires<Arc<dyn OutputProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn OutputProvider + Send + Sync> { Arc::new(self.clone()) }
}
