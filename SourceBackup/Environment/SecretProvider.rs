// @module SecretProvider (Environment)
// @description Implements the `SecretsProvider` trait for
// `MountainEnvironment`.

use std::sync::Arc;

use async_trait::async_trait;
use Common::{Environment::Requires, error::CommonError, secret::SecretsProvider};

use super::MountainEnvironment;
use crate::Handler::secret as SecretHandler;

#[async_trait]
impl SecretsProvider for MountainEnvironment {
	async fn GetSecret(&self, extension_identifier:String, key:String) -> Result<Option<String>, CommonError> {
		SecretHandler::GetSecretLogic(&self.ApplicationHandle, extension_identifier, key).await
	}

	async fn StoreSecret(&self, extension_identifier:String, key:String, value:String) -> Result<(), CommonError> {
		SecretHandler::StoreSecretLogic(&self.ApplicationHandle, extension_identifier, key, value).await
	}

	async fn DeleteSecret(&self, extension_identifier:String, key:String) -> Result<(), CommonError> {
		SecretHandler::DeleteSecretLogic(&self.ApplicationHandle, extension_identifier, key).await
	}
}

impl Requires<Arc<dyn SecretsProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn SecretsProvider + Send + Sync> { Arc::new(self.clone()) }
}
