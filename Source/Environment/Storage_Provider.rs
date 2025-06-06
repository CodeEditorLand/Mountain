// ---------------------------------------------------------------------------------------------
// Mountain Environment - Storage Provider 
// --------------------------------------------------------------------------------------------
// This module implements the `StorageProvider` trait for `MountainEnvironment`.
// It provides access to persistent key-value storage, similar to VS Code's
// Memento API, scoped globally or to the current workspace. Operations are
// delegated to handler functions in `handlers::storage`.
// --------------------------------------------------------------------------------------------

use std::sync::Arc;

use Land_Common::{
	environment::Requires,
	errors::CommonError,
	storage_effects::StorageProvider, // The trait being implemented
};
use async_trait::async_trait;
use log::{info, trace}; // For logging
use serde_json::Value;

use crate::{
	environment::MountainEnvironment,
	handlers, // For delegating to storage handlers
};

// --- StorageProvider Implementation ---
#[async_trait]
impl StorageProvider for MountainEnvironment {
	async fn get_storage_value(&self, is_global_scope:bool, key:&str) -> Result<Option<Value>, CommonError> {
		trace!("[Env StorageProv] GetValue: scope_is_global={}, key='{}'", is_global_scope, key);

		// Delegate to the handler function.
		handlers::storage::handle_get_storage_value_effect_logic(self.app_handle.clone(), is_global_scope, key).await
	}

	async fn update_storage_value(
		&self,
		is_global_scope:bool,
		key:String,
		value_to_set:Option<Value>, // Some(Value) to set/update, None to delete
	) -> Result<(), CommonError> {
		info!(
			"[Env StorageProv] UpdateValue: scope_is_global={}, key='{}', value_is_some={}",
			is_global_scope,
			key,
			value_to_set.is_some()
		);

		// Delegate to the handler function.
		handlers::storage::handle_set_storage_value_effect_logic(
			self.app_handle.clone(),
			is_global_scope,
			key,
			value_to_set,
		)
		.await
	}
}

// --- Requires Implementation ---
impl Requires<Arc<dyn StorageProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn StorageProvider + Send + Sync> { Arc::new(self.clone()) }
}
