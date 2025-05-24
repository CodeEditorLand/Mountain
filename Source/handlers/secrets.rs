// ---------------------------------------------------------------------------------------------
// Mountain Secrets Handlers (handlers/secrets.rs)
// --------------------------------------------------------------------------------------------
// Provides the backend implementation for the `vscode.SecretStorage` API,

// handling secure storage and retrieval of sensitive extension data using the
// operating system's native keychain or credential store.
//
// Responsibilities:
// - Handling `$getPassword`, `$setPassword`, `$deletePassword` RPC calls
//   proxied from Cocoon's `secret-state-shim.js` (via Track/effects).
// - Interacting with the `keyring` crate.
// - Constructing service/username keys for `keyring`.
// - Mapping `keyring::Error` to structured error messages/codes.
//
// Key Interactions:
// - Called by effects created in `track.rs` or directly by RPC dispatcher.
// - Uses the `keyring` crate.
// - Needs `AppHandle` for app's bundle ID to create unique service names.
// --------------------------------------------------------------------------------------------

// Import the Entry type
use keyring::Entry;
// Added trace
use log::{error, info, trace, warn};
use serde_json::{Value, json};
// Added Manager for app.config()
use tauri::{AppHandle, Manager, Runtime};

// Use shared error utilities
use crate::handlers::error_utils;

// Map keyring errors to a structured error string using shared utilities
fn map_keyring_error_to_rpc_str(e:keyring::Error, operation:&str, key_context:&str) -> String {
	let error_message_prefix = format!("Keyring operation '{}' for key context '{}' failed", operation, key_context);

	// Log the original keyring error with context
	error!("{}: {}", error_message_prefix, e);

	let (specific_message, code_str) = match e.kind() {
		keyring::ErrorKind::NoEntry => (format!("{}: Secret not found.", error_message_prefix), "ESECRET_NOENTRY"),

		keyring::ErrorKind::Ambiguous => {
			(
				format!(
					"{}: Ambiguous result, multiple entries found (unexpected).",
					error_message_prefix
				),
				"ESECRET_AMBIGUOUS",
			)
		},

		keyring::ErrorKind::BadEncoding(_) => {
			(
				format!("{}: Data encoding/decoding error.", error_message_prefix),
				"ESECRET_ENCODING",
			)
		},

		keyring::ErrorKind::InvalidAppId => {
			(
				format!(
					"{}: Invalid application identifier configuration for keyring.",
					error_message_prefix
				),
				"ESECRET_APPID",
			)
		},

		keyring::ErrorKind::InvalidServiceName(_) => {
			(
				format!("{}: Invalid service name configuration for keyring.", error_message_prefix),
				"ESECRET_SERVICENAME",
			)
		},

		keyring::ErrorKind::PlatformFailure(_) => {
			(
				format!(
					"{}: Underlying OS platform failure for keyring operation.",
					error_message_prefix
				),
				"ESECRET_PLATFORM",
			)
		},

		keyring::ErrorKind::NoBackend => {
			(
				format!("{}: No suitable OS keychain/credential backend found.", error_message_prefix),
				"ESECRET_NOBACKEND",
			)
		},

		keyring::ErrorKind::BadPassword => {
			(
				format!("{}: Incorrect password or permission issue with keyring.", error_message_prefix),
				"ESECRET_PERM",
			)
		},

		keyring::ErrorKind::Duplicate => {
			(
				format!("{}: Attempted to create a duplicate secret entry.", error_message_prefix),
				"ESECRET_DUP",
			)
		},

		keyring::ErrorKind::Cancelled => {
			(
				format!("{}: Keyring operation cancelled by user or system.", error_message_prefix),
				"ESECRET_CANCELLED",
			)
		},

		// Catch-all for other kinds
		_ => {
			(
				format!("{}: Unknown keyring error occurred: {}", error_message_prefix, e),
				"ESECRET_UNKNOWN",
			)
		},
	};

	error_utils::rpc_error_string(specific_message, Some(code_str))
}

// Get service name for keyring entry (must be consistent)
fn get_keyring_service_name<R:Runtime>(app:&AppHandle<R>, extension_id:&str) -> String {
	// Use App bundle identifier + extension ID for uniqueness and stability
	let app_bundle_id = app.config().tauri.bundle.identifier.clone();

	format!("{}.{}", app_bundle_id, extension_id)
}

pub async fn handle_get_secret<R:Runtime>(app:AppHandle<R>, params:Value) -> Result<Value, String> {
	let extension_id = params
		.get("extensionId")
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("get_secret", "extensionId", "string", None))?;

	let key = params
		.get("key")
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("get_secret", "key", "string", None))?;

	// Use trace for potentially frequent calls
	trace!("[Secrets Handler] GetSecret ext='{}', key='{}'", extension_id, key);

	let service_name = get_keyring_service_name(&app, extension_id);

	let key_context_for_error = format!("ext: '{}', key: '{}'", extension_id, key);

	let entry = Entry::new(&service_name, key)
		.map_err(|e| map_keyring_error_to_rpc_str(e, "entry creation (get)", &key_context_for_error))?;

	match entry.get_password() {
		Ok(password) => {
			trace!("[Secrets Handler] Secret found for ext='{}', key='{}'", extension_id, key);

			// Return password string as JSON
			Ok(json!(password))
		},

		Err(keyring::Error { kind: keyring::ErrorKind::NoEntry, .. }) => {
			trace!("[Secrets Handler] Secret not found for ext='{}', key='{}'", extension_id, key);

			// Key not found is not an error for get, return null
			Ok(Value::Null)
		},

		Err(e) => Err(map_keyring_error_to_rpc_str(e, "get_password", &key_context_for_error)),
	}
}

pub async fn handle_store_secret<R:Runtime>(app:AppHandle<R>, params:Value) -> Result<Value, String> {
	let extension_id = params
		.get("extensionId")
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("store_secret", "extensionId", "string", None))?;

	let key = params
		.get("key")
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("store_secret", "key", "string", None))?;

	// Value MUST be a string for keyring
	let value = params
		.get("value")
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("store_secret", "value", "string", None))?;

	if value.is_empty() {
		warn!(
			"[Secrets Handler] Storing empty string for secret ext='{}', key='{}'. This might behave unexpectedly on \
			 some OS keyring backends.",
			extension_id, key
		);
	}

	// Info as it's a modification
	info!("[Secrets Handler] StoreSecret ext='{}', key='{}'", extension_id, key);

	let service_name = get_keyring_service_name(&app, extension_id);

	let key_context_for_error = format!("ext: '{}', key: '{}'", extension_id, key);

	let entry = Entry::new(&service_name, key)
		.map_err(|e| map_keyring_error_to_rpc_str(e, "entry creation (store)", &key_context_for_error))?;

	entry.set_password(value)
		 // Return JSON null on success
		.map(|_| Value::Null)
		.map_err(|e| map_keyring_error_to_rpc_str(e, "set_password", &key_context_for_error))
}

pub async fn handle_delete_secret<R:Runtime>(app:AppHandle<R>, params:Value) -> Result<Value, String> {
	let extension_id = params
		.get("extensionId")
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("delete_secret", "extensionId", "string", None))?;

	let key = params
		.get("key")
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("delete_secret", "key", "string", None))?;

	// Info as it's a modification
	info!("[Secrets Handler] DeleteSecret ext='{}', key='{}'", extension_id, key);

	let service_name = get_keyring_service_name(&app, extension_id);

	let key_context_for_error = format!("ext: '{}', key: '{}'", extension_id, key);

	let entry = Entry::new(&service_name, key)
		.map_err(|e| map_keyring_error_to_rpc_str(e, "entry creation (delete)", &key_context_for_error))?;

	match entry.delete_password() {
		Ok(_) => {
			info!(
				"[Secrets Handler] Secret deleted successfully for ext='{}', key='{}'",
				extension_id, key
			);

			Ok(Value::Null)
		},

		Err(keyring::Error { kind: keyring::ErrorKind::NoEntry, .. }) => {
			info!(
				"[Secrets Handler] Secret not found for deletion (ext='{}', key='{}'), considered success.",
				extension_id, key
			);

			// OK if not found
			Ok(Value::Null)
		},

		Err(e) => Err(map_keyring_error_to_rpc_str(e, "delete_password", &key_context_for_error)),
	}
}
