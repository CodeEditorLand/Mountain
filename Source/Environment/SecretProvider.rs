//! # SecretProvider Implementation
//!
//! Implements the `SecretProvider` trait for the `MountainEnvironment`. This
//! provider contains the core logic for secure secret storage using the system
//! keyring, powered by the `keyring` crate.

use Common::{Error::CommonError::CommonError, Secret::SecretProvider::SecretProvider};
use async_trait::async_trait;
use keyring::Entry;
use log::{info, trace};

use super::MountainEnvironment::MountainEnvironment;

/// Constructs the service name for the keyring entry.
fn GetKeyringServiceName(Environment:&MountainEnvironment, ExtensionIdentifier:&str) -> String {
	format!("{}.{}", Environment.ApplicationHandle.package_info().name, ExtensionIdentifier)
}

#[async_trait]
impl SecretProvider for MountainEnvironment {
	/// Retrieves a secret by reading from the OS keychain.
	async fn GetSecret(&self, ExtensionIdentifier:String, Key:String) -> Result<Option<String>, CommonError> {
		trace!(
			"[SecretProvider] Getting secret for ext: '{}', key: '{}'",
			ExtensionIdentifier, Key
		);
		let ServiceName = GetKeyringServiceName(self, &ExtensionIdentifier);
		let Entry = Entry::new(&ServiceName, &Key)
			.map_err(|e| CommonError::SecretsAccess { Key:Key.clone(), Reason:e.to_string() })?;

		match Entry.get_password() {
			Ok(Password) => Ok(Some(Password)),
			Err(keyring::Error::NoEntry) => Ok(None),
			Err(e) => Err(CommonError::SecretsAccess { Key, Reason:e.to_string() }),
		}
	}

	/// Stores a secret by writing to the OS keychain.
	async fn StoreSecret(&self, ExtensionIdentifier:String, Key:String, Value:String) -> Result<(), CommonError> {
		info!(
			"[SecretProvider] Storing secret for ext: '{}', key: '{}'",
			ExtensionIdentifier, Key
		);
		let ServiceName = GetKeyringServiceName(self, &ExtensionIdentifier);
		let Entry = Entry::new(&ServiceName, &Key)
			.map_err(|e| CommonError::SecretsAccess { Key:Key.clone(), Reason:e.to_string() })?;

		Entry
			.set_password(&Value)
			.map_err(|e| CommonError::SecretsAccess { Key, Reason:e.to_string() })
	}

	/// Deletes a secret by removing it from the OS keychain.
	async fn DeleteSecret(&self, ExtensionIdentifier:String, Key:String) -> Result<(), CommonError> {
		info!(
			"[SecretProvider] Deleting secret for ext: '{}', key: '{}'",
			ExtensionIdentifier, Key
		);
		let ServiceName = GetKeyringServiceName(self, &ExtensionIdentifier);
		let Entry = Entry::new(&ServiceName, &Key)
			.map_err(|e| CommonError::SecretsAccess { Key:Key.clone(), Reason:e.to_string() })?;

		match Entry.delete_credential() {
			Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
			Err(e) => Err(CommonError::SecretsAccess { Key, Reason:e.to_string() }),
		}
	}
}
