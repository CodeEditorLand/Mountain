// @module SyncProvider (Environment)
// @description Implements the `SyncProvider` trait for `MountainEnvironment`.

use std::sync::Arc;

use async_trait::async_trait;
use Common::{Environment::Requires, error::CommonError, sync::SyncProvider};
use log::warn;
use serde_json::Value;

use super::MountainEnvironment;

#[async_trait]
impl SyncProvider for MountainEnvironment {
	async fn PushUserData(&self, _user_data:Value) -> Result<(), CommonError> {
		warn!("[SyncProvider] PushUserData is not implemented.");
		// A real implementation would connect to a settings sync service.
		Ok(())
	}

	async fn PullUserData(&self) -> Result<Value, CommonError> {
		warn!("[SyncProvider] PullUserData is not implemented.");
		Ok(Value::Null)
	}
}

impl Requires<Arc<dyn SyncProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn SyncProvider + Send + Sync> { Arc::new(self.clone()) }
}
