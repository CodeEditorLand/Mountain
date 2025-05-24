// ---------------------------------------------------------------------------------------------
// Mountain Secrets Handlers (handlers/secrets.rs)
// --------------------------------------------------------------------------------------------
// Provides the backend implementation for the `vscode.SecretStorage` API,

// handling secure storage and retrieval of sensitive extension data using the
// operating system's native keychain or credential store (e.g., macOS Keychain,

// Windows Credential Manager, Linux Secret Service API).
//
// Responsibilities:
// - Handling `$getPassword`, `$setPassword`, `$deletePassword` RPC calls
//   proxied from Cocoon's `secret-state-shim.js`. These calls are typically
//   transformed into effects by `track.rs` and then executed by the
//   `SecretsProvider` implementation in `environment.rs`, which in turn calls
//   these handlers.
// - Interacting with the `keyring` crate to perform OS-level secure storage
//   operations.
// - Constructing a unique service name for `keyring` entries, typically by
//   combining the application's bundle identifier and the extension's ID, to
//   namespace secrets appropriately.
// - Mapping `keyring::Error` types to structured JSON-RPC error strings with
//   specific codes (e.g., `ESECRET_NOENTRY`) for consistent error reporting
//   back to Cocoon.
//
// Key Interactions:
// - Called by `environment.rs` (implementing `SecretsProvider` trait methods)
//   which are invoked by effects created in `track.rs`.
// - Uses the `keyring::Entry` API for get, set, and delete operations.
// - Requires `AppHandle` to access the application's bundle identifier from
//   Tauri configuration for constructing service names.
// - Uses `handlers::error_utils` for formatting RPC error responses.
// --------------------------------------------------------------------------------------------

// Import the Entry type for keyring operations
use keyring::Entry;
// For logging operations and errors
use log::{error, info, trace, warn};
use serde_json::{Value, json};
// Manager for app.config(), Runtime for generic AppHandle
use tauri::{AppHandle, Manager, Runtime};

// Use shared error utilities
use crate::handlers::error_utils;

/// Maps errors from the `keyring` crate to a structured JSON-RPC error string.
///
/// This function provides detailed error messages and specific error codes
/// based on the `keyring::ErrorKind`.
///
/// # Arguments
/// * `e` - The `keyring::Error` to map.
/// * `operation` - A string describing the keyring operation being attempted
///   (e.g., "get_password", "entry creation").
/// * `key_context` - A string providing context about the key being accessed
///   (e.g., "ext: 'publisher.name', key: 'apiToken'").
///
/// # Returns
/// A `String` containing the JSON-formatted RPC error.
fn map_keyring_error_to_rpc_string(e:keyring::Error, operation:&str, key_context:&str) -> String {
	let error_message_prefix = format!("Keyring operation '{}' for {} failed", operation, key_context);

	// Log the original, detailed keyring error internally for debugging.
	error!("{}: {}", error_message_prefix, e);

	let (specific_message, code_str) = match e.kind() {
		keyring::ErrorKind::NoEntry => {
			(
				format!("{}: Secret not found in keychain.", error_message_prefix),
				"ESECRET_NOENTRY",
			)
		},

		keyring::ErrorKind::Ambiguous => {
			(
				format!(
					"{}: Ambiguous result; multiple entries found (this is unexpected for extension secrets).",
					error_message_prefix
				),
				"ESECRET_AMBIGUOUS",
			)
		},

		keyring::ErrorKind::BadEncoding(_) => {
			(
				format!("{}: Data encoding or decoding error with keychain.", error_message_prefix),
				"ESECRET_ENCODING",
			)
		},

		keyring::ErrorKind::InvalidAppId => {
			(
				format!(
					"{}: Invalid application identifier configuration for keyring access.",
					error_message_prefix
				),
				"ESECRET_APPID",
			)
		},

		keyring::ErrorKind::InvalidServiceName(_) => {
			(
				format!(
					"{}: Invalid service name configuration for keyring access.",
					error_message_prefix
				),
				"ESECRET_SERVICENAME",
			)
		},

		keyring::ErrorKind::PlatformFailure(_) => {
			(
				format!(
					"{}: Underlying OS platform failure during keyring operation.",
					error_message_prefix
				),
				"ESECRET_PLATFORM",
			)
		},

		keyring::ErrorKind::NoBackend => {
			(
				format!(
					"{}: No suitable OS keychain or credential backend could be found or accessed.",
					error_message_prefix
				),
				"ESECRET_NOBACKEND",
			)
		},

		keyring::ErrorKind::BadPassword => {
			(
				// This might mean the keychain is locked or requires user authentication.
				format!(
					"{}: Incorrect password or permission issue accessing the OS keyring/keychain.",
					error_message_prefix
				),
				"ESECRET_PERM",
			)
		},

		keyring::ErrorKind::Duplicate => {
			(
				format!(
					"{}: Attempted to create a duplicate secret entry where one already exists.",
					error_message_prefix
				),
				"ESECRET_DUP",
			)
		},

		keyring::ErrorKind::Cancelled => {
			(
				format!(
					"{}: Keyring operation was cancelled by the user or system (e.g., a prompt was dismissed).",
					error_message_prefix
				),
				"ESECRET_CANCELLED",
			)
		},

		_ => {
			(
				// Catch-all for other/new keyring error kinds
				format!(
					"{}: An unknown or unspecified keyring error occurred: {}",
					error_message_prefix, e
				),
				"ESECRET_UNKNOWN",
			)
		},
	};

	error_utils::rpc_error_string(specific_message, Some(code_str))
}

/// Constructs the service name used for storing secrets in the OS keychain.
///
/// The service name is namespaced using the application's bundle identifier
/// and the extension's ID to ensure uniqueness and prevent collisions between
/// extensions or other applications.
///
/// # Arguments
/// * `app` - The Tauri `AppHandle` to access `app.config()`.
/// * `extension_id` - The identifier of the extension (e.g., "publisher.name").
///
/// # Returns
/// A `String` representing the service name (e.g.,
///
/// "com.example.landeditor.publisher.myextension").
fn get_keyring_service_name_for_extension<R:Runtime>(app:&AppHandle<R>, extension_id:&str) -> String {
	// Use the app's bundle identifier (e.g., "com.example.landeditor") as a prefix.
	let app_bundle_id = app.config().tauri.bundle.identifier.clone();

	// Append the extension ID to create a unique service name.
	format!("{}.{}", app_bundle_id, extension_id)
	// TODO: Consider sanitizing extension_id if it can contain characters
	// problematic for service names on some platforms.       The `keyring`
	// crate might handle some sanitization internally, but it's good to be
	// aware.
}

/// Handles the `$getPassword` RPC request (via `SecretsProvider` effect).
///
/// Retrieves a secret for a given extension and key from the OS keychain.
///
/// # Arguments
/// * `app` - The Tauri `AppHandle`.
/// * `params` - A `serde_json::Value` object: `{ "extensionId": string, "key":
///   string }`
///
/// # Returns
/// * `Ok(Value::String)` containing the secret if found.
/// * `Ok(Value::Null)` if the secret is not found (this is not an error for
///   `get`).
/// * `Err(String)` with a JSON-RPC error if parameters are invalid or a keyring
///   error occurs.
pub async fn handle_get_secret<R:Runtime>(app:AppHandle<R>, params:Value) -> Result<Value, String> {
	let extension_id = params
		.get("extensionId")
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("get_secret", "extensionId", "string", None))?;

	let key = params
		.get("key")
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("get_secret", "key", "string", None))?;

	// Use trace for potentially frequent calls like get.
	trace!("[Secrets Handler] GetSecret: extension_id='{}', key='{}'", extension_id, key);

	let service_name = get_keyring_service_name_for_extension(&app, extension_id);

	let key_context_for_error_log = format!("extensionId: '{}', key: '{}'", extension_id, key);

	// Create a keyring Entry for the specified service and username (key).
	let entry = Entry::new(&service_name, key).map_err(|e| {
		map_keyring_error_to_rpc_string(e, "keyring entry creation (for get)", &key_context_for_error_log)
	})?;

	match entry.get_password() {
		Ok(password) => {
			trace!(
				"[Secrets Handler] Secret found for extension_id='{}', key='{}'",
				extension_id, key
			);

			// Return password string as JSON Value::String
			Ok(json!(password))
		},

		Err(keyring::Error { kind: keyring::ErrorKind::NoEntry, .. }) => {
			trace!(
				"[Secrets Handler] Secret not found for extension_id='{}', key='{}' (NoEntry). Returning null.",
				extension_id, key
			);

			// Key not found is a valid outcome for `get`, return null as per VS Code API.
			Ok(Value::Null)
		},

		Err(e) => {
			// Other keyring errors during get_password.
			Err(map_keyring_error_to_rpc_string(e, "get_password", &key_context_for_error_log))
		},
	}
}

/// Handles the `$setPassword` RPC request (via `SecretsProvider` effect).
///
/// Stores or updates a secret for a given extension and key in the OS keychain.
///
/// # Arguments
/// * `app` - The Tauri `AppHandle`.
/// * `params` - A `serde_json::Value` object: `{ "extensionId": string, "key":
///   string, "value": string }`
///
/// # Returns
/// * `Ok(Value::Null)` on successful storage.
/// * `Err(String)` with a JSON-RPC error if parameters are invalid or a keyring
///   error occurs.
pub async fn handle_store_secret<R:Runtime>(app:AppHandle<R>, params:Value) -> Result<Value, String> {
	let extension_id = params
		.get("extensionId")
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("store_secret", "extensionId", "string", None))?;

	let key = params
		.get("key")
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("store_secret", "key", "string", None))?;

	// Value to store MUST be a string for keyring.
	let value_to_store = params
		.get("value")
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("store_secret", "value", "string", None))?;

	// Log as info because it's a modification operation.
	info!("[Secrets Handler] StoreSecret: extension_id='{}', key='{}'", extension_id, key);

	// Avoid logging the actual secret value unless at trace level and truncated.
	trace!(
		"[Secrets Handler] Value to store (first 10 chars): '{}...'",
		value_to_store.chars().take(10).collect::<String>()
	);

	if value_to_store.is_empty() {
		warn!(
			"[Secrets Handler] Storing an empty string for secret: extension_id='{}', key='{}'. Some OS keyring \
			 backends might treat this differently (e.g., delete or error).",
			extension_id, key
		);
	}

	let service_name = get_keyring_service_name_for_extension(&app, extension_id);

	let key_context_for_error_log = format!("extensionId: '{}', key: '{}'", extension_id, key);

	let entry = Entry::new(&service_name, key).map_err(|e| {
		map_keyring_error_to_rpc_string(e, "keyring entry creation (for store)", &key_context_for_error_log)
	})?;

	entry
		.set_password(value_to_store)
		.map(|_| {
			info!(
				"[Secrets Handler] Secret stored successfully for extension_id='{}', key='{}'",
				extension_id, key
			);

			// Return JSON null on success
			Value::Null
		})
		.map_err(|e| map_keyring_error_to_rpc_string(e, "set_password", &key_context_for_error_log))
}

/// Handles the `$deletePassword` RPC request (via `SecretsProvider` effect).
///
/// Deletes a secret for a given extension and key from the OS keychain.
///
/// # Arguments
/// * `app` - The Tauri `AppHandle`.
/// * `params` - A `serde_json::Value` object: `{ "extensionId": string, "key":
///   string }`
///
/// # Returns
/// * `Ok(Value::Null)` on successful deletion or if the secret was not found
///   (idempotent).
/// * `Err(String)` with a JSON-RPC error if parameters are invalid or a keyring
///   error occurs (other than `NoEntry`).
pub async fn handle_delete_secret<R:Runtime>(app:AppHandle<R>, params:Value) -> Result<Value, String> {
	let extension_id = params
		.get("extensionId")
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("delete_secret", "extensionId", "string", None))?;

	let key = params
		.get("key")
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("delete_secret", "key", "string", None))?;

	// Log as info because it's a modification operation.
	info!("[Secrets Handler] DeleteSecret: extension_id='{}', key='{}'", extension_id, key);

	let service_name = get_keyring_service_name_for_extension(&app, extension_id);

	let key_context_for_error_log = format!("extensionId: '{}', key: '{}'", extension_id, key);

	let entry = Entry::new(&service_name, key).map_err(|e| {
		map_keyring_error_to_rpc_string(e, "keyring entry creation (for delete)", &key_context_for_error_log)
	})?;

	match entry.delete_password() {
		Ok(_) => {
			info!(
				"[Secrets Handler] Secret deleted successfully for extension_id='{}', key='{}'",
				extension_id, key
			);

			Ok(Value::Null)
		},

		Err(keyring::Error { kind: keyring::ErrorKind::NoEntry, .. }) => {
			info!(
				"[Secrets Handler] Secret not found for deletion (extension_id='{}', key='{}'). Considered success \
				 (idempotent).",
				extension_id, key
			);

			// Deleting a non-existent secret is not an error; it's idempotent.
			Ok(Value::Null)
		},

		Err(e) => {
			Err(map_keyring_error_to_rpc_string(
				e,
				"delete_password",
				&key_context_for_error_log,
			))
		},
	}
}
