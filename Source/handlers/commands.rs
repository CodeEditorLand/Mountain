// ---------------------------------------------------------------------------------------------
// Mountain Command Handlers (handlers/commands.rs)
// --------------------------------------------------------------------------------------------
// Implements the core logic for managing and executing commands within
// Mountain, handling interactions originating from both the frontend (via
// Track) and sidecars (via RPC/Vine/Track). It maintains the central command
// registry.
//
// Responsibilities:
// - Managing the command registry (stored in `AppState`) which tracks both
//   native Mountain commands and commands registered by sidecars (like Cocoon).
// - Handling `$registerCommand` RPC calls from Cocoon: Registers a *proxy*
//   handler that forwards execution back to the originating sidecar via Vine.
// - Handling `$unregisterCommand` RPC calls from Cocoon: Removes the
//   corresponding proxy handler if the requesting sidecar is the owner.
// - Handling `$getCommands` RPC calls from Cocoon: Returns a combined list of
//   all registered native and proxied command IDs.
// - Handling command execution requests (`handle_execute_command`, called by
//   Track):
//   - Looks up the command ID in the registry.
//   - If native: Executes the corresponding native logic (calling the stored
//     handler function).
//   - If proxied: Sends a request (`commands_executeContributedCommand`) via
//     Vine to the owning sidecar to execute the command there.
//   - Returns the result or error from the execution.
// - Providing functions to register native commands during Mountain startup
//   (`register_native_command_internal`).
// - Implementing the actual logic for native commands (e.g.,

//   `handle_native_save_all`).
//
// Key Interactions:
// - Interacts heavily with `AppState` to access/modify the `command_registry`.
// - Called by `track::dispatch_sidecar_request` for RPC methods.
// - Called by `track::dispatch_command` (indirectly via effect) for execution.
// - Uses `vine::send_request` to proxy command execution back to sidecars.
// - Executes native command logic directly or via `ActionEffect`s using
//   `AppRuntime`.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,

	future::Future,

	pin::Pin,

	// Use standard Mutex
	sync::{Arc, Mutex as StdMutex, MutexGuard},
};

// Assume Land_Common provides necessary effects if used by native handlers
use Land_Common::{command_effects, ui_effects, workspace_effects};
// Use log crate for logging
use log;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime as TauriRuntime, State, Window};

use crate::{
	// Import AppState and CommandHandler enum
	app_state::{AppState, CommandHandler},

	runtime::AppRuntime,

	// Track might not be needed directly if only called by it
	track,

	// Vine required for proxying back to sidecars
	vine,
};

// --- Helper Functions ---

/// Helper to create a structured error JSON string for handler results.
fn create_handler_error_string(message:String, code:Option<&str>) -> String {
	json!({ "message": message, "code": code.unwrap_or("EUNKNOWN") }).to_string()
}

/// Helper to map Mutex lock poisoning errors to the handler's error string
/// format.
fn map_lock_error<T>(e:std::sync::PoisonError<MutexGuard<'_, T>>) -> String {
	create_handler_error_string(format!("Failed to acquire lock on command registry: {}", e), Some("ELOCKED"))
}

// --- Request Handlers (Called by Track dispatcher for sidecar requests) ---

/// Handles the `commands_registerCommand` request from a sidecar shim.
/// Stores the association between the sidecar and the registered command ID.
pub async fn handle_register_command<R:TauriRuntime>(
	app:AppHandle<R>,

	sidecar_id:String,

	params:Value,
) -> Result<Value, String> {
	let command_id = params
		.get("id")
		.and_then(|v| v.as_str())
		.ok_or_else(|| create_handler_error_string("Missing or invalid 'id' parameter".to_string(), Some("EBADARG")))?
		.to_string();

	// Keep: Essential registration action log
	log::info!("[Cmd Handler] Registering PROXY for '{}' from '{}'", command_id, sidecar_id);

	let app_state = app.state::<AppState>();

	let mut registry = app_state.command_registry.lock().map_err(map_lock_error)?;

	// Keep: Warning for overwrite is useful
	if registry.contains_key(&command_id) {
		log::warn!(
			"[Cmd Handler] Warning: Command ID '{}' is already registered. Overwriting.",
			command_id
		);

		// TODO: Add ownership tracking and update/remove old owner if
		// overwriting
	}

	// Insert the proxy handler
	registry.insert(
		command_id.clone(),
		CommandHandler::Proxied { sidecar_id:sidecar_id.clone(), command_id:command_id.clone() },
	);

	// Keep: Confirmation of success
	log::info!("[Cmd Handler] Command '{}' registered successfully in AppState.", command_id);

	// TODO: Register ownership if implemented

	// Success, void operation
	Ok(Value::Null)
}

/// Handles the `commands_unregisterCommand` request from a sidecar shim.
/// Removes the command ID association for the originating sidecar (if owned).
pub async fn handle_unregister_command<R:TauriRuntime>(
	app:AppHandle<R>,

	sidecar_id:String,

	params:Value,
) -> Result<Value, String> {
	let command_id = params
		.get("id")
		.and_then(|v| v.as_str())
		.ok_or_else(|| create_handler_error_string("Missing or invalid 'id' parameter".to_string(), Some("EBADARG")))?;

	// Keep: Essential unregistration action log
	log::info!(
		"[Cmd Handler] Unregistering command '{}' from sidecar '{}'",
		command_id,
		sidecar_id
	);

	let app_state = app.state::<AppState>();

	let mut registry = app_state.command_registry.lock().map_err(map_lock_error)?;

	// TODO: Implement ownership check before removing.
	// For now, simple removal:
	// Keep: Confirmation or warning log
	if registry.remove(command_id).is_some() {
		log::info!("[Cmd Handler] Command '{}' unregistered.", command_id);

		// TODO: Also remove from ownership map if implemented
	} else {
		log::warn!("[Cmd Handler] Command '{}' not found for unregistration.", command_id);
	}

	// Success, void operation even if not found
	Ok(Value::Null)
}

/// Handles the `commands_getCommands` request from a sidecar shim.
/// Returns a list of *all* registered command IDs (native and proxied).
pub async fn handle_get_commands<R:TauriRuntime>(
	app:AppHandle<R>,

	// Keep runtime state injection if needed later
	_runtime:State<'_, Arc<AppRuntime>>,
) -> Result<Value, String> {
	// Reduced logging: log::trace!("[Cmd Handler] Handling getCommands request");

	let app_state = app.state::<AppState>();

	let registry = app_state.command_registry.lock().map_err(map_lock_error)?;

	let command_list:Vec<String> = registry.keys().cloned().collect();

	// Release lock
	drop(registry);

	Ok(json!(command_list))
}

/// Handles the `commands_executeCommand` request, typically from a sidecar shim
/// or the frontend. Determines if the command is native or registered by a
/// sidecar and routes accordingly.
pub async fn handle_execute_command<R:TauriRuntime>(
	app:AppHandle<R>,

	// Window context might be needed for native actions
	window:Window<R>,

	// Runtime needed for executing native effects
	runtime:State<'_, Arc<AppRuntime>>,

	// Expects { "id": "command.id", "args": [...] }
	params:Value,
) -> Result<Value, String> {
	let command_id = params
		.get("id")
		.and_then(|v| v.as_str())
		.ok_or_else(|| create_handler_error_string("Missing or invalid 'id' parameter".to_string(), Some("EBADARG")))?
		.to_string();

	// Default to Null if missing
	let args = params.get("args").cloned().unwrap_or(Value::Null);

	// Keep: Essential action log
	log::info!("[Cmd Handler] Handling executeCommand '{}'", command_id);

	let app_state = app.state::<AppState>();

	// Get handler info while holding lock briefly
	let handler_info = {
		let registry = app_state.command_registry.lock().map_err(map_lock_error)?;

		registry.get(&command_id).cloned()
		// Lock released
	};

	match handler_info {
		Some(CommandHandler::Native(native_handler_fn)) => {
			// Keep: Distinguishes native execution path
			log::debug!("[Cmd Handler] Found Native handler for '{}'. Executing...", command_id);

			native_handler_fn(app, window, runtime.inner().clone(), args).await
		},

		Some(CommandHandler::Proxied { sidecar_id, command_id: proxied_cmd_id }) => {
			// Keep: Distinguishes proxied execution path
			log::debug!(
				"[Cmd Handler] Found Proxied handler for '{}'. Routing execution to sidecar '{}'",
				proxied_cmd_id,
				sidecar_id
			);

			let request_payload = json!({ "id": proxied_cmd_id, "args": args });

			let sidecar_method = "commands_executeContributedCommand".to_string();

			vine::send_request(&sidecar_id, sidecar_method, request_payload, false, 30000)
				.await
				.map_err(|e| {
					log::error!(
						"[Cmd Handler] Vine error routing command '{}' to sidecar '{}': {}",
						command_id,
						sidecar_id,
						e
					);

					create_handler_error_string(
						format!("Failed to route command '{}' to sidecar '{}'", command_id, sidecar_id),
						Some("EIPC"),
					)
				})
		},

		None => {
			// Keep: Important error log
			log::error!("[Cmd Handler] Command '{}' not found in registry.", command_id);

			// Return structured error string
			Err(create_handler_error_string(
				format!("Command '{}' not found.", command_id),
				Some("ENOCMD"),
				// Example custom code
			))
		},
	}
}

// --- Native Command Handler Implementations ---

/// Native handler for `workbench.action.files.saveAll`. Runs the effect.
pub fn handle_native_save_all<R:TauriRuntime>(
	_app:AppHandle<R>,

	_window:Window<R>,

	runtime:Arc<AppRuntime>,

	args:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		let include_untitled = args.get(0).and_then(|v| v.as_bool()).unwrap_or(true);

		log::info!("[Native Cmd] Executing saveAll (includeUntitled: {})", include_untitled);

		let effect = workspace_effects::save_all(include_untitled);

		runtime.run(effect).await.map(|_| Value::Null).map_err(|e| e.to_string())
	})
}

/// Native handler for `mountain.action.showAbout`. Runs the effect.
pub fn handle_native_show_about<R:TauriRuntime>(
	_app:AppHandle<R>,

	_window:Window<R>,

	runtime:Arc<AppRuntime>,

	_args:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		log::info!("[Native Cmd] Executing showAbout");

		let message = format!("Land Editor (Mountain)\nVersion: {}\nMore info...", env!("CARGO_PKG_VERSION"));

		let effect = ui_effects::show_message("info".to_string(), message, None);

		runtime.run(effect).await.map(|_| Value::Null).map_err(|e| e.to_string())
	})
}

/// Placeholder native handler function.
pub fn native_placeholder_handler<R:TauriRuntime>(
	_app:AppHandle<R>,

	_window:Window<R>,

	_runtime:Arc<AppRuntime>,

	args:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		// Keep: Useful for seeing if placeholder was called
		log::info!("[Native Cmd Placeholder] Executed with args: {:?}", args);

		Ok(json!("Native Placeholder OK"))
	})
}

// --- Native Command Registration Helper ---

/// Helper to register a native command handler map during startup (e.g., in
/// `app_state.rs`).
pub fn register_native_command_internal<R:TauriRuntime + 'static>(
	registry:&mut HashMap<String, CommandHandler<R>>,

	command_id:String,

	handler:fn(
		AppHandle<R>,

		Window<R>,

		Arc<AppRuntime>,

		Value,
	) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>,
) {
	// Keep: Warning for collisions
	if registry.contains_key(&command_id) {
		log::warn!(
			"[Cmd Registry Init] Warning: Native command ID '{}' collision during registration.",
			command_id
		);
	}

	// Keep: Log registration during init
	log::info!("[Cmd Registry Init] Registered native command: {}", command_id);

	registry.insert(command_id, CommandHandler::Native(handler));
}
