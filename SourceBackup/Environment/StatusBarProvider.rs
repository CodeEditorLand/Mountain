// @module StatusBarProvider (Environment)
// @description Implements the `StatusBarProvider` trait for
// `MountainEnvironment` by delegating to logic Handler and making RPC calls
// for dynamic data.

use std::sync::Arc;

use async_trait::async_trait;
use Common::{
	Environment::Requires,
	error::CommonError,
	status_bar::{StatusBarProvider, DTO::*},
};
use serde_json::{Value, json};

use super::MountainEnvironment;
use crate::{Handler::status_bar as StatusBarHandler, Vine::client as VineClient};

#[async_trait]
impl StatusBarProvider for MountainEnvironment {
	/// Handle a request to create or update a status bar entry by delegating
	/// to the `StatusBarHandler`.
	async fn SetEntry(&self, entry:StatusBarEntryDto) -> Result<(), CommonError> {
		StatusBarHandler::SetEntryLogic(&self.ApplicationHandle, entry).await
	}

	/// Handle a request to dispose of a status bar entry by delegating to the
	/// `StatusBarHandler`.
	async fn DisposeEntry(&self, entry_id:String) -> Result<(), CommonError> {
		StatusBarHandler::DisposeEntryLogic(&self.ApplicationHandle, entry_id).await
	}

	/// Handle a request to resolve a dynamic tooltip.
	///
	/// This is a "reverse" call, where the host (`Mountain`) needs to get data
	/// from the extension host (`Cocoon`). It makes a gRPC call to the
	/// sidecar and returns the result.
	async fn ProvideTooltip(&self, entry_id:String) -> Result<Option<Value>, CommonError> {
		let rpc_response = VineClient::SendRequest(
			"cocoon-main".to_string(),
			"$provideStatusbarTooltip".to_string(),
			json!([entry_id]),
			5000,
		) // 5-second timeout
		.await?;

		// If the response is null or fails to parse, we gracefully return None.
		Ok(serde_json::from_value(rpc_response).unwrap_or(None))
	}
}

impl Requires<Arc<dyn StatusBarProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn StatusBarProvider + Send + Sync> { Arc::new(self.clone()) }
}
