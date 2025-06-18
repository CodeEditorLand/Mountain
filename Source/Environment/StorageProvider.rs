// @module StorageProvider (Environment)
// @description Implements the `StorageProvider` trait for
// `MountainEnvironment`.

use std::sync::Arc;

use async_trait::async_trait;
use Common::{Environment::Requires, error::CommonError, storage::StorageProvider};
use serde_json::Value;

use super::MountainEnvironment;
use crate::Handler::storage as StorageHandler;

#[async_trait]
impl StorageProvider for MountainEnvironment {
	async fn GetStorageValue(&self, is_global_scope:bool, key:&str) -> Result<Option<Value>, CommonError> {
		StorageHandler::GetStorageValueLogic(&self.ApplicationHandle, is_global_scope, key).await
	}

	async fn UpdateStorageValue(
		&self,
		is_global_scope:bool,
		key:String,
		value_to_set:Option<Value>,
	) -> Result<(), CommonError> {
		StorageHandler::UpdateStorageValueLogic(&self.ApplicationHandle, is_global_scope, key, value_to_set).await
	}
}

impl Requires<Arc<dyn StorageProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn StorageProvider + Send + Sync> { Arc::new(self.clone()) }
}
