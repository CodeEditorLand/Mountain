// File: Handlers/Secrets/Secrets.rs
// Contains the primary logic for handling secret storage operations,
// using the system keyring for secure data persistence.

#![allow(non_snake_case, non_camel_case_types)]

use keyring::Entry;
use log::{error, info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};

use crate::Handlers::ErrorUtils;

/// Maps a `keyring::Error` to a standardized RPC error string.
fn MapKeyringErrorToRpcString(KeyringError:keyring::Error, Operation:&str, KeyContext:&str) -> String {
	let ErrorMessagePrefix = format!("Keyring operation '{}' for {} failed", Operation, KeyContext);
	error!("{}: {}", ErrorMessagePrefix, KeyringError);

	let (SpecificMessage, CodeString) = match KeyringError {
		keyring::Error::NoEntry => {
			(
				format!("{}: Secret not found in keychain.", ErrorMessagePrefix),
				"ESECRET_NOENTRY",
			)
		},
		keyring::Error::Ambiguous(_) => {
			(
				format!("{}: Ambiguous result; multiple entries found.", ErrorMessagePrefix),
				"ESECRET_AMBIGUOUS",
			)
		},
		keyring::Error::BadEncoding(_) => {
			(
				format!("{}: Data encoding or decoding error with keychain.", ErrorMessagePrefix),
				"ESECRET_ENCODING",
			)
		},
		keyring::Error::Invalid(..) => {
			(
				format!("{}: Invalid application identifier for keyring access.", ErrorMessagePrefix),
				"ESECRET_APPID",
			)
		},
		keyring::Error::InvalidServiceName(_) => {
			(
				format!("{}: Invalid service name for keyring access.", ErrorMessagePrefix),
				"ESECRET_SERVICENAME",
			)
		},
		keyring::Error::PlatformFailure(_) => {
			(
				format!(
					"{}: Underlying OS platform failure during keyring operation.",
					ErrorMessagePrefix
				),
				"ESECRET_PLATFORM",
			)
		},
		keyring::Error::NoStorageAccess(_) => {
			(
				format!(
					"{}: No suitable OS keychain or credential backend found or accessible.",
					ErrorMessagePrefix
				),
				"ESECRET_NOBACKEND",
			)
		},
		// Note: The original had two PlatformFailure matches. Consolidating.
		_ => {
			(
				format!(
					"{}: An unknown or unspecified keyring error occurred: {}",
					ErrorMessagePrefix, KeyringError
				),
				"ESECRET_UNKNOWN",
			)
		},
	};

	ErrorUtils::RpcErrorString(SpecificMessage, Some(CodeString))
}

/// Constructs the service name for the keyring entry, namespacing it to the
/// application and extension.
fn GetKeyringServiceNameForExtension<R:Runtime>(ApplicationHandle:&AppHandle<R>, ExtensionIdentifier:&str) -> String {
	let ApplicationBundleIdentifier = ApplicationHandle.config().identifier.clone();
	format!("{}.{}", ApplicationBundleIdentifier, ExtensionIdentifier)
}

/// The core logic for handling the `get_secret` effect.
pub async fn HandleGetSecretEffectLogic<R:Runtime>(
	ApplicationHandle:AppHandle<R>,
	ExtensionIdentifier:String,
	Key:String,
) -> Result<Option<String>, crate::Errors::CommonError> {
	// Using CommonError for effect logic
	trace!(
		"[SecretsHandler Logic] GetSecret: ExtensionIdentifier='{}', Key='{}'",
		ExtensionIdentifier, Key
	);
	let ServiceName = GetKeyringServiceNameForExtension(&ApplicationHandle, &ExtensionIdentifier);
	let Entry = Entry::new(&ServiceName, &Key).map_err(|KeyringError| {
		// Map keyring error to a CommonError variant
		crate::Errors::CommonError::SecretsAccess { Key:Key.clone(), Reason:KeyringError.to_string() }
	})?;

	match Entry.get_password() {
		Ok(Password) => Ok(Some(Password)),
		Err(keyring::Error::NoEntry) => Ok(None),
		Err(KeyringError) => Err(crate::Errors::CommonError::SecretsAccess { Key, Reason:KeyringError.to_string() }),
	}
}

/// The core logic for handling the `store_secret` effect.
pub async fn HandleStoreSecretEffectLogic<R:Runtime>(
	ApplicationHandle:AppHandle<R>,
	ExtensionIdentifier:String,
	Key:String,
	ValueToStore:String,
) -> Result<(), crate::Errors::CommonError> {
	info!(
		"[SecretsHandler Logic] StoreSecret: ExtensionIdentifier='{}', Key='{}'",
		ExtensionIdentifier, Key
	);
	let ServiceName = GetKeyringServiceNameForExtension(&ApplicationHandle, &ExtensionIdentifier);
	let Entry = Entry::new(&ServiceName, &Key).map_err(|KeyringError| {
		crate::Errors::CommonError::SecretsAccess { Key:Key.clone(), Reason:KeyringError.to_string() }
	})?;
	Entry
		.set_password(&ValueToStore)
		.map_err(|KeyringError| crate::Errors::CommonError::SecretsAccess { Key, Reason:KeyringError.to_string() })
}

/// The core logic for handling the `delete_secret` effect.
pub async fn HandleDeleteSecretEffectLogic<R:Runtime>(
	ApplicationHandle:AppHandle<R>,
	ExtensionIdentifier:String,
	Key:String,
) -> Result<(), crate::Errors::CommonError> {
	info!(
		"[SecretsHandler Logic] DeleteSecret: ExtensionIdentifier='{}', Key='{}'",
		ExtensionIdentifier, Key
	);
	let ServiceName = GetKeyringServiceNameForExtension(&ApplicationHandle, &ExtensionIdentifier);
	let Entry = Entry::new(&ServiceName, &Key).map_err(|KeyringError| {
		crate::Errors::CommonError::SecretsAccess { Key:Key.clone(), Reason:KeyringError.to_string() }
	})?;
	match Entry.delete_password() {
		Ok(_) => Ok(()),
		Err(keyring::Error::NoEntry) => Ok(()), // Deleting a non-existent secret is a success (idempotent).
		Err(KeyringError) => Err(crate::Errors::CommonError::SecretsAccess { Key, Reason:KeyringError.to_string() }),
	}
}
