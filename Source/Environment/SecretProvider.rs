//! # SecretProvider Implementation
//!
//! Implements the `SecretProvider` trait for the `MountainEnvironment`. This
//! provider contains the core logic for secure secret storage using the system
//! keyring, powered by the `keyring` crate.

use Common::{Error::CommonError::CommonError, Secret::SecretProvider};
use async_trait::async_trait;
use keyring::Entry;
use log::{info, trace};
use tauri::Manager;

use super::MountainEnvironment::MountainEnvironment;

/// Constructs the service name for the keyring entry.
///
/// This is a crucial security feature that namespaces secrets on a
/// per-extension basis, preventing one extension from reading another's
/// secrets. It combines the application's unique identifier with the
/// extension's identifier.
fn GetKeyringServiceName(Environment:&MountainEnvironment, ExtensionIdentifier:&str) -> String {
	format!("{}.{}", Environment.ApplicationHandle.config().identifier, ExtensionIdentifier)
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
			Err(keyring::Error::NoEntry) => Ok(None), // Not finding a secret is not an error.
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

		// This operation is idempotent; it is considered successful even if the
		// entry doesn't exist.
		match Entry.delete_password() {
			Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
			Err(e) => Err(CommonError::SecretsAccess { Key, Reason:e.to_string() }),
		}
	}
}
