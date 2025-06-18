// @module SecretsLogic
// @description Contains the core logic for secure secret storage using the
// system keyring, powered by the `keyring` crate.

use Common::error::CommonError;
use keyring::Entry;
use log::{info, trace};
use tauri::{AppHandle, Manager, Runtime};

// Constructs the service name for the keyring entry.
//
// This is a crucial security feature that namespaces secrets on a
// per-extension basis, preventing one extension from reading another's
// secrets. It typically combines the application identifier (e.g.,
// `com.land.mountain`) with the extension's identifier (e.g.,
// `github.copilot`).
fn get_keyring_service_name<R:Runtime>(app_handle:&AppHandle<R>, extension_identifier:&str) -> String {
	format!("{}.{}", app_handle.config().identifier, extension_identifier)
}

// Logic for handling the `GetSecret` effect by reading from the OS keychain.
pub async fn GetSecretLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	extension_identifier:String,
	key:String,
) -> Result<Option<String>, CommonError> {
	trace!(
		"[SecretsLogic] Getting secret for ext: '{}', key: '{}'",
		extension_identifier, key
	);
	let service_name = get_keyring_service_name(app_handle, &extension_identifier);
	let entry = Entry::new(&service_name, &key)
		.map_err(|e| CommonError::SecretsAccess { Key:key.clone(), Reason:e.to_string() })?;

	match entry.get_password() {
		Ok(password) => Ok(Some(password)),
		Err(keyring::Error::NoEntry) => Ok(None), // Secret not found is not an error.
		Err(e) => Err(CommonError::SecretsAccess { Key:key, Reason:e.to_string() }),
	}
}

// Logic for handling the `StoreSecret` effect by writing to the OS keychain.
pub async fn StoreSecretLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	extension_identifier:String,
	key:String,
	value:String,
) -> Result<(), CommonError> {
	info!(
		"[SecretsLogic] Storing secret for ext: '{}', key: '{}'",
		extension_identifier, key
	);
	let service_name = get_keyring_service_name(app_handle, &extension_identifier);
	let entry = Entry::new(&service_name, &key)
		.map_err(|e| CommonError::SecretsAccess { Key:key.clone(), Reason:e.to_string() })?;

	entry
		.set_password(&value)
		.map_err(|e| CommonError::SecretsAccess { Key:key, Reason:e.to_string() })
}

// Logic for handling the `DeleteSecret` effect by removing from the OS
// keychain.
pub async fn DeleteSecretLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	extension_identifier:String,
	key:String,
) -> Result<(), CommonError> {
	info!(
		"[SecretsLogic] Deleting secret for ext: '{}', key: '{}'",
		extension_identifier, key
	);
	let service_name = get_keyring_service_name(app_handle, &extension_identifier);
	let entry = Entry::new(&service_name, &key)
		.map_err(|e| CommonError::SecretsAccess { Key:key.clone(), Reason:e.to_string() })?;

	// The operation is considered successful even if the entry doesn't exist.
	match entry.delete_password() {
		Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
		Err(e) => Err(CommonError::SecretsAccess { Key:key, Reason:e.to_string() }),
	}
}
