// @module TerminalProvider (Environment)
// @description Implements the `TerminalProvider` trait for
// `MountainEnvironment` by delegating to the logic Handler in
// `Handler::terminal`.

use std::sync::Arc;

use async_trait::async_trait;
use Common::{Environment::Requires, error::CommonError, terminal::TerminalProvider};
use serde_json::Value;

use super::MountainEnvironment;
use crate::Handler::terminal as TerminalHandler;

#[async_trait]
impl TerminalProvider for MountainEnvironment {
	// Handle the creation of a new terminal by delegating to the
	// `TerminalHandler`.
	async fn CreateTerminal(&self, options_value:Value) -> Result<Value, CommonError> {
		TerminalHandler::CreateTerminalLogic(&self.ApplicationHandle, options_value).await
	}

	// Handle sending text to a terminal by delegating to the
	// `TerminalHandler`.
	async fn SendTextToTerminal(&self, terminal_id:u64, text:String) -> Result<(), CommonError> {
		TerminalHandler::SendTextToTerminalLogic(&self.ApplicationHandle, terminal_id, text).await
	}

	// Handle disposing of a terminal by delegating to the `TerminalHandler`.
	async fn DisposeTerminal(&self, terminal_id:u64) -> Result<(), CommonError> {
		TerminalHandler::DisposeTerminalLogic(&self.ApplicationHandle, terminal_id).await
	}
}

impl Requires<Arc<dyn TerminalProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn TerminalProvider + Send + Sync> { Arc::new(self.clone()) }
}
