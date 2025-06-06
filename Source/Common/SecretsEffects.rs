
// Defines the SecretsProvider trait and associated effects for securely
// storing and retrieving sensitive data like API keys or tokens.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use async_trait::async_trait;

use crate::{
	Effect::ActionEffect,
	Environment::{Environment, Requires},
	Errors::CommonError,
	Runtime::AppRuntimeTrait,
};

/// A trait for environments that can interact with a secure credential store
/// (e.g., system keyring).
#[async_trait]
pub trait SecretsProvider: Environment {
	/// Retrieves a secret for a given extension and key.
	async fn GetSecret(&self, ExtensionIdentifier:String, Key:String) -> Result<Option<String>, CommonError>;
	/// Stores a secret for a given extension and key.
	async fn StoreSecret(&self, ExtensionIdentifier:String, Key:String, Value:String) -> Result<(), CommonError>;
	/// Deletes a secret for a given extension and key.
	async fn DeleteSecret(&self, ExtensionIdentifier:String, Key:String) -> Result<(), CommonError>;
}

/// Creates an effect to retrieve a secret.
pub fn GetSecret<RuntimeAccessType>(
	ExtensionIdentifier:String,
	Key:String,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Option<String>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn SecretsProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let ExtensionIdentifierClone = ExtensionIdentifier.clone();
		let KeyClone = Key.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn SecretsProvider> = Environment.require();
			Provider.GetSecret(ExtensionIdentifierClone, KeyClone).await
		})
	}))
}

/// Creates an effect to store a secret.
pub fn StoreSecret<RuntimeAccessType>(
	ExtensionIdentifier:String,
	Key:String,
	Value:String,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn SecretsProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let ExtensionIdentifierClone = ExtensionIdentifier.clone();
		let KeyClone = Key.clone();
		let ValueClone = Value.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn SecretsProvider> = Environment.require();
			Provider.StoreSecret(ExtensionIdentifierClone, KeyClone, ValueClone).await
		})
	}))
}

/// Creates an effect to delete a secret.
pub fn DeleteSecret<RuntimeAccessType>(
	ExtensionIdentifier:String,
	Key:String,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn SecretsProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let ExtensionIdentifierClone = ExtensionIdentifier.clone();
		let KeyClone = Key.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn SecretsProvider> = Environment.require();
			Provider.DeleteSecret(ExtensionIdentifierClone, KeyClone).await
		})
	}))
}
