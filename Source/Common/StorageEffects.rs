
// Defines the StorageProvider trait and associated effects for interacting with
// Memento-style storage (both global and workspace-scoped).

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::{
	Effect::ActionEffect,
	Environment::{Environment, Requires},
	Errors::CommonError,
	Runtime::AppRuntimeTrait,
};

/// A trait for environments that can provide key-value storage capabilities.
#[async_trait]
pub trait StorageProvider: Environment {
	/// Retrieves a value from storage based on scope and key.
	async fn GetStorageValue(&self, IsGlobalScope:bool, Key:&str) -> Result<Option<Value>, CommonError>;
	/// Updates or deletes a value in storage based on scope and key.
	/// Setting `ValueToSet` to `None` should delete the key.
	async fn UpdateStorageValue(
		&self,
		IsGlobalScope:bool,
		Key:String,
		ValueToSet:Option<Value>,
	) -> Result<(), CommonError>;
}

/// Creates an effect to retrieve a value from storage.
/// `TargetObject` is a JSON Value expected to contain `scope` (bool) and `key`
/// (string).
pub fn GetStorageItem<RuntimeAccessType>(
	TargetObject:Value,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Option<Value>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn StorageProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let TargetObjectClone = TargetObject.clone();
		Box::pin(async move {
			let IsGlobal = TargetObjectClone.get("scope").and_then(Value::as_bool).unwrap_or(false);
			let KeyString = TargetObjectClone
				.get("key")
				.and_then(Value::as_str)
				.ok_or_else(|| {
					CommonError::InvalidArg {
						ArgumentName:"TargetObject.key".to_string(),
						Reason:"Expected a 'key' string field in TargetObject.".to_string(),
					}
				})?
				.to_string();
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn StorageProvider> = Environment.require();
			Provider.GetStorageValue(IsGlobal, &KeyString).await
		})
	}))
}

/// Creates an effect to set or delete a value in storage.
/// `TargetObject` is a JSON Value expected to contain `scope` (bool) and `key`
/// (string). If `ValueToSet` is `Value::Null`, the effect should delete the
/// item.
pub fn SetStorageItem<RuntimeAccessType>(
	TargetObject:Value,
	ValueToSet:Value,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn StorageProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let TargetObjectClone = TargetObject.clone();
		let ValueToSetClone = ValueToSet.clone();
		Box::pin(async move {
			let IsGlobal = TargetObjectClone.get("scope").and_then(Value::as_bool).unwrap_or(false);
			let KeyString = TargetObjectClone
				.get("key")
				.and_then(Value::as_str)
				.ok_or_else(|| {
					CommonError::InvalidArg {
						ArgumentName:"TargetObject.key".to_string(),
						Reason:"Expected a 'key' string field in TargetObject.".to_string(),
					}
				})?
				.to_string();
			let ValueOption = if ValueToSetClone.is_null() { None } else { Some(ValueToSetClone) };
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn StorageProvider> = Environment.require();
			Provider.UpdateStorageValue(IsGlobal, KeyString, ValueOption).await
		})
	}))
}
