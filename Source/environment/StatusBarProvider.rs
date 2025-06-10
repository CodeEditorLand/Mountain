use std::sync::Arc;

use Common::{
	environment::Requires,
	error::CommonError,
	status_bar::{StatusBarProvider, dto::*},
};
use async_trait::async_trait;
use serde_json::{Value, json};

/// @module StatusBarProvider (Environment)
/// @description Implements the `StatusBarProvider` trait for
/// `MountainEnvironment` by delegating to logic handlers and making RPC calls
/// for dynamic data.
use super::MountainEnvironment;
use crate::{handlers::status_bar as StatusBarHandler, vine::client as VineClient};

#[async_trait]
impl StatusBarProvider for MountainEnvironment {
	/// Handles a request to create or update a status bar entry by delegating
	/// to the `StatusBarHandler`.
	async fn SetEntry(&self, Entry:StatusBarEntryDto) -> Result<(), CommonError> {
		StatusBarHandler::SetEntryLogic(&self.AppHandle, Entry).await
	}

	/// Handles a request to dispose of a status bar entry by delegating to the
	/// `StatusBarHandler`.
	async fn DisposeEntry(&self, EntryId:String) -> Result<(), CommonError> {
		StatusBarHandler::DisposeEntryLogic(&self.AppHandle, EntryId).await
	}

	/// Handles a request to resolve a dynamic tooltip.
	///
	/// This is a "reverse" call, where the host (`Mountain`) needs to get data
	/// from the extension host (`Cocoon`). It makes a gRPC call to the
	/// sidecar and returns the result.
	async fn ProvideTooltip(&self, EntryId:String) -> Result<Option<Value>, CommonError> {
		let RpcResponse = VineClient::SendRequest(
			"cocoon-main".to_string(),
			"$provideStatusbarTooltip".to_string(),
			json!([EntryId]),
			5000, // 5-second timeout
		)
		.await?;

		// If the response is null or fails to parse, we gracefully return None.
		Ok(serde_json::from_value(RpcResponse).unwrap_or(None))
	}
}

impl Requires<Arc<dyn StatusBarProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn StatusBarProvider + Send + Sync> { Arc::new(self.clone()) }
}
