// ---------------------------------------------------------------------------------------------
// Mountain Environment - Secrets Provider 
// --------------------------------------------------------------------------------------------
// This module implements the `SecretsProvider` trait for `MountainEnvironment`.
// It handles secure storage and retrieval of sensitive information (secrets),
// typically using the operating system's keyring or equivalent secure storage.
// Operations are delegated to handler functions in `handlers::secrets`.
// --------------------------------------------------------------------------------------------

use std::sync::Arc;

use Land_Common::{
	environment::Requires,
	errors::CommonError,
	secrets_effects::SecretsProvider, // The trait being implemented
};
use async_trait::async_trait;
use log::{info, trace}; // For logging

use crate::{
	environment::MountainEnvironment,
	handlers, // For delegating to secrets handlers
};

// --- SecretsProvider Implementation ---
#[async_trait]
impl SecretsProvider for MountainEnvironment {
	async fn get_secret(&self, extension_id:String, key:String) -> Result<Option<String>, CommonError> {
		trace!("[Env SecretsProv] GetSecret: extension_id='{}', key='{}'", extension_id, key);

		// Delegate to the handler function.
		// The handler in `handlers::secrets` will interact with the keyring.
		handlers::secrets::handle_get_secret_effect_logic(self.app_handle.clone(), extension_id, key).await
	}

	async fn store_secret(&self, extension_id:String, key:String, value_to_store:String) -> Result<(), CommonError> {
		info!("[Env SecretsProv] StoreSecret: extension_id='{}', key='{}'", extension_id, key);
		// Value is not logged for security.

		// Delegate to the handler function.
		handlers::secrets::handle_store_secret_effect_logic(self.app_handle.clone(), extension_id, key, value_to_store)
			.await
	}

	async fn delete_secret(&self, extension_id:String, key:String) -> Result<(), CommonError> {
		info!("[Env SecretsProv] DeleteSecret: extension_id='{}', key='{}'", extension_id, key);

		// Delegate to the handler function.
		handlers::secrets::handle_delete_secret_effect_logic(self.app_handle.clone(), extension_id, key).await
	}
}

// --- Requires Implementation ---
impl Requires<Arc<dyn SecretsProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn SecretsProvider + Send + Sync> { Arc::new(self.clone()) }
}
