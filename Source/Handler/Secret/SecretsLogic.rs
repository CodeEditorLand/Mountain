use Common::error::CommonError;
use keyring::Entry;
use log::{info, trace};
use tauri::{ApplicationHandle, Manager, RunTime};

// @module SecretsLogic
// @description Contains the core logic for secure secret storage using the
// system keyring, powered by the `keyring` crate.
use crate::Handler::error_utils;

// Constructs the service name for the keyring entry.
//
// This is a crucial security feature that namespaces secrets on a
// per-extension basis, preventing one extension from reading another's
// secrets. It typically combines the application identifier (e.g.,
// `com.land.mountain`) with the extension's identifier (e.g.,
// `github.copilot`).
fn GetKeyringServiceName<R:RunTime>(ApplicationHandle:&ApplicationHandle<R>, ExtensionIdentifier:&str) -> String {
	format!("{}.{}", ApplicationHandle.config().identifier, ExtensionIdentifier)
}

// Logic for handling the `GetSecret` effect by reading from the OS keychain.
pub async fn GetSecretLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	ExtensionIdentifier:String,
	Key:String,
) -> Result<Option<String>, CommonError> {
	trace!(
		"[SecretsLogic] Getting secret for ext: '{}', key: '{}'",
		ExtensionIdentifier, Key
	);
	let ServiceName = GetKeyringServiceName(ApplicationHandle, &ExtensionIdentifier);
	let Entry = Entry::new(&ServiceName, &Key)
		.map_err(|e| CommonError::SecretsAccess { Key:Key.clone(), Reason:e.to_string() })?;

	match Entry.get_password() {
		Ok(Password) => Ok(Some(Password)),
		Err(keyring::Error::NoEntry) => Ok(None), // Secret not found is not an error.
		Err(e) => Err(CommonError::SecretsAccess { Key, Reason:e.to_string() }),
	}
}

// Logic for handling the `StoreSecret` effect by writing to the OS keychain.
pub async fn StoreSecretLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	ExtensionIdentifier:String,
	Key:String,
	Value:String,
) -> Result<(), CommonError> {
	info!(
		"[SecretsLogic] Storing secret for ext: '{}', key: '{}'",
		ExtensionIdentifier, Key
	);
	let ServiceName = GetKeyringServiceName(ApplicationHandle, &ExtensionIdentifier);
	let Entry = Entry::new(&ServiceName, &Key)
		.map_err(|e| CommonError::SecretsAccess { Key:Key.clone(), Reason:e.to_string() })?;

	Entry
		.set_password(&Value)
		.map_err(|e| CommonError::SecretsAccess { Key, Reason:e.to_string() })
}

// Logic for handling the `DeleteSecret` effect by removing from the OS
// keychain.
pub async fn DeleteSecretLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	ExtensionIdentifier:String,
	Key:String,
) -> Result<(), CommonError> {
	info!(
		"[SecretsLogic] Deleting secret for ext: '{}', key: '{}'",
		ExtensionIdentifier, Key
	);
	let ServiceName = GetKeyringServiceName(ApplicationHandle, &ExtensionIdentifier);
	let Entry = Entry::new(&ServiceName, &Key)
		.map_err(|e| CommonError::SecretsAccess { Key:Key.clone(), Reason:e.to_string() })?;

	// The operation is considered successful even if the entry doesn't exist.
	match Entry.delete_password() {
		Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
		Err(e) => Err(CommonError::SecretsAccess { Key, Reason:e.to_string() }),
	}
}
