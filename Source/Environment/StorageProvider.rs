
// Implements the `StorageProvider` trait for the `MountainEnvironment`.
// This file connects abstract storage effects to the concrete logic
// in the application's storage handlers for Memento management.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{Environment::Requires, Errors::CommonError, StorageEffect::StorageProvider};
use async_trait::async_trait;
use log::{info, trace};
use serde_json::Value;

use crate::{Environment::MountainEnvironment, Handlers};

#[async_trait]
impl StorageProvider for MountainEnvironment {
	/// Retrieves a value from either global or workspace storage.
	async fn GetStorageValue(&self, IsGlobalScope:bool, Key:&str) -> Result<Option<Value>, CommonError> {
		let ScopeName = if IsGlobalScope { "Global" } else { "Workspace" };
		trace!("[Environment StorageProvider] GetValue: Scope='{}', Key='{}'", ScopeName, Key);
		Handlers::Storage::HandleGetStorageValueEffectLogic(self.AppHandle.clone(), IsGlobalScope, Key).await
	}

	/// Updates or deletes a value in either global or workspace storage.
	async fn UpdateStorageValue(
		&self,
		IsGlobalScope:bool,
		Key:String,
		ValueToSet:Option<Value>,
	) -> Result<(), CommonError> {
		let ScopeName = if IsGlobalScope { "Global" } else { "Workspace" };
		info!(
			"[Environment StorageProvider] UpdateValue: Scope='{}', Key='{}', ValueIsSome={}",
			ScopeName,
			Key,
			ValueToSet.is_some()
		);
		Handlers::Storage::HandleSetStorageValueEffectLogic(self.AppHandle.clone(), IsGlobalScope, Key, ValueToSet)
			.await
	}
}

impl Requires<Arc<dyn StorageProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn StorageProvider + Send + Sync> { Arc::new(self.clone()) }
}
