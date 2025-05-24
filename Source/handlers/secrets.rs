// ---------------------------------------------------------------------------------------------
// Mountain Secrets Handlers (handlers/secrets.rs)
// --------------------------------------------------------------------------------------------
// Provides the backend implementation for the `vscode.SecretStorage` API,
// handling secure storage and retrieval of sensitive extension data using the
// operating system's native keychain or credential store.
//
// Responsibilities:
// - Handling `$getPassword`, `$setPassword`, `$deletePassword` RPC calls
//   proxied from Cocoon's `secret-state-shim.js`.
// - Interacting with the `keyring` crate to securely store, retrieve, and
//   delete secrets.
// - Constructing appropriate service/username keys for the `keyring` crate,
//   typically based on the application identifier and the extension ID, to
//   ensure isolation.
// - Mapping `keyring::Error` types to structured error messages/codes suitable
//   for returning to the Cocoon shim.
//
// Key Interactions:
// - Called by `track::dispatch_sidecar_request` (or secrets effects) for RPC
//   methods.
// - Uses the `keyring` crate for OS credential store interaction.
// - Needs `AppHandle` to construct unique service names based on the app's
//   bundle ID.
// --------------------------------------------------------------------------------------------

// ----- START: Element/Mountain/src/handlers/secrets.rs -----
use keyring::Entry; // Import the Entry type
use serde_json::{Value, json};
use tauri::{AppHandle, Runtime};

// Helper to create structured error string
fn create_error_string(message:String, code:Option<&str>) -> String {
	json!({"message": message, "code": code.unwrap_or("EUNKNOWN")}).to_string()
}

// Map keyring errors to error strings
fn map_keyring_error(e:keyring::Error, operation:&str) -> String {
	let code = match e {
		keyring::Error::NoEntry => "ENOENT",
		keyring::Error::Ambiguous => "EAMBIGUOUS", // Example custom code
		keyring::Error::BadEncoding(_) => "EBADENCODING",
		keyring::Error::InvalidAppId | keyring::Error::InvalidServiceName(_) => "EINVALIDAPPID",
		keyring::Error::PlatformFailure(_) => "EPLATFORM",
		keyring::Error::NoBackend => "ENOBACKEND",
		keyring::Error::BadPassword => "EBADPASS", // Maybe permission related?
		_ => "EKEYRING",                           // Generic
	};
	create_error_string(format!("Keyring {} error: {}", operation, e), Some(code))
}

// Get service name for keyring entry (must be consistent)
fn get_keyring_service_name<R:Runtime>(app:&AppHandle<R>, extension_id:&str) -> String {
	// Use App bundle identifier + extension ID for uniqueness
	// Make sure package_info() is available and correct.
	let app_name = app.package_info().name.clone();
	format!("{}.{}", app_name, extension_id)
}

pub async fn handle_get_secret<R:Runtime>(app:AppHandle<R>, params:Value) -> Result<Value, String> {
	let extension_id = params
		.get("extensionId")
		.and_then(|v| v.as_str())
		.ok_or_else(|| create_error_string("Missing extensionId".to_string(), Some("EBADARG")))?;
	let key = params
		.get("key")
		.and_then(|v| v.as_str())
		.ok_or_else(|| create_error_string("Missing key".to_string(), Some("EBADARG")))?;
	println!("[Secrets Handler] GetSecret ext={}, key={}", extension_id, key);

	let service = get_keyring_service_name(&app, extension_id);
	let entry = Entry::new(&service, key); // Use key as username/account

	match entry.get_password() {
		Ok(password) => Ok(json!(password)),             // Return password string as JSON
		Err(keyring::Error::NoEntry) => Ok(Value::Null), // Key not found, return null
		Err(e) => Err(map_keyring_error(e, "get")),
	}
}

pub async fn handle_store_secret<R:Runtime>(app:AppHandle<R>, params:Value) -> Result<Value, String> {
	let extension_id = params
		.get("extensionId")
		.and_then(|v| v.as_str())
		.ok_or_else(|| create_error_string("Missing extensionId".to_string(), Some("EBADARG")))?;
	let key = params
		.get("key")
		.and_then(|v| v.as_str())
		.ok_or_else(|| create_error_string("Missing key".to_string(), Some("EBADARG")))?;
	// Value MUST be a string for keyring
	let value = params
		.get("value")
		.and_then(|v| v.as_str())
		.ok_or_else(|| create_error_string("Missing or invalid value (must be string)".to_string(), Some("EBADARG")))?;
	println!("[Secrets Handler] StoreSecret ext={}, key={}", extension_id, key);

	let service = get_keyring_service_name(&app, extension_id);
	let entry = Entry::new(&service, key);

	entry
		.set_password(value)
		.map(|_| Value::Null) // Return JSON null on success
		.map_err(|e| map_keyring_error(e, "store"))
}

pub async fn handle_delete_secret<R:Runtime>(app:AppHandle<R>, params:Value) -> Result<Value, String> {
	let extension_id = params
		.get("extensionId")
		.and_then(|v| v.as_str())
		.ok_or_else(|| create_error_string("Missing extensionId".to_string(), Some("EBADARG")))?;
	let key = params
		.get("key")
		.and_then(|v| v.as_str())
		.ok_or_else(|| create_error_string("Missing key".to_string(), Some("EBADARG")))?;
	println!("[Secrets Handler] DeleteSecret ext={}, key={}", extension_id, key);

	let service = get_keyring_service_name(&app, extension_id);
	let entry = Entry::new(&service, key);

	match entry.delete_password() {
		Ok(_) => Ok(Value::Null),
		Err(keyring::Error::NoEntry) => Ok(Value::Null), // OK if not found
		Err(e) => Err(map_keyring_error(e, "delete")),
	}
}
// ----- END: Element/Mountain/src/handlers/secrets.rs -----
