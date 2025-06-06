// File: Environment/SecretsProvider.rs
// Implements the `SecretsProvider` trait for the `MountainEnvironment`.
// This file connects abstract secrets effects to the concrete logic
// in the application's secrets handlers, which use the system keyring.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{Environment::Requires, Errors::CommonError, SecretsEffect::SecretsProvider};
use async_trait::async_trait;
use log::{info, trace};

use crate::{Environment::MountainEnvironment, Handlers};

#[async_trait]
impl SecretsProvider for MountainEnvironment {
	/// Retrieves a secret from the system's secure storage.
	async fn GetSecret(&self, ExtensionIdentifier:String, Key:String) -> Result<Option<String>, CommonError> {
		trace!(
			"[Environment SecretsProvider] GetSecret: ExtensionIdentifier='{}', Key='{}'",
			ExtensionIdentifier, Key
		);
		Handlers::Secrets::HandleGetSecretEffectLogic(self.AppHandle.clone(), ExtensionIdentifier, Key).await
	}

	/// Stores a secret in the system's secure storage.
	async fn StoreSecret(
		&self,
		ExtensionIdentifier:String,
		Key:String,
		ValueToStore:String,
	) -> Result<(), CommonError> {
		info!(
			"[Environment SecretsProvider] StoreSecret: ExtensionIdentifier='{}', Key='{}'",
			ExtensionIdentifier, Key
		);
		Handlers::Secrets::HandleStoreSecretEffectLogic(self.AppHandle.clone(), ExtensionIdentifier, Key, ValueToStore)
			.await
	}

	/// Deletes a secret from the system's secure storage.
	async fn DeleteSecret(&self, ExtensionIdentifier:String, Key:String) -> Result<(), CommonError> {
		info!(
			"[Environment SecretsProvider] DeleteSecret: ExtensionIdentifier='{}', Key='{}'",
			ExtensionIdentifier, Key
		);
		Handlers::Secrets::HandleDeleteSecretEffectLogic(self.AppHandle.clone(), ExtensionIdentifier, Key).await
	}
}

impl Requires<Arc<dyn SecretsProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn SecretsProvider + Send + Sync> { Arc::new(self.clone()) }
}
