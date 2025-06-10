use std::sync::Arc;

use Common::{environment::Requires, error::CommonError, terminal::TerminalProvider};
use async_trait::async_trait;
use serde_json::Value;

/// @module TerminalProvider (Environment)
/// @description Implements the `TerminalProvider` trait for
/// `MountainEnvironment` by delegating to the logic handlers in
/// `handlers::terminal`.
use super::MountainEnvironment;
use crate::handlers::terminal as TerminalHandler;

#[async_trait]
impl TerminalProvider for MountainEnvironment {
	/// Handles the creation of a new terminal by delegating to the
	/// `TerminalHandler`.
	async fn CreateTerminal(&self, OptionsValue:Value) -> Result<Value, CommonError> {
		// The handler logic returns a Result<Value, String>, so we map the error
		// into our more structured CommonError type.
		TerminalHandler::CreateTerminalLogic(&self.AppHandle, OptionsValue)
			.await
			.map_err(|e_str| CommonError::IpcError { Description:e_str })
	}

	/// Handles sending text to a terminal by delegating to the
	/// `TerminalHandler`.
	async fn SendTextToTerminal(&self, TerminalId:u64, Text:String) -> Result<(), CommonError> {
		TerminalHandler::SendTextToTerminalLogic(&self.AppHandle, TerminalId, Text)
			.await
			.map_err(|e_str| CommonError::IpcError { Description:e_str })
	}

	/// Handles disposing of a terminal by delegating to the `TerminalHandler`.
	async fn DisposeTerminal(&self, TerminalId:u64) -> Result<(), CommonError> {
		TerminalHandler::DisposeTerminalLogic(&self.AppHandle, TerminalId)
			.await
			.map_err(|e_str| CommonError::IpcError { Description:e_str })
	}
}

impl Requires<Arc<dyn TerminalProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn TerminalProvider + Send + Sync> { Arc::new(self.clone()) }
}
