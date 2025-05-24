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
//   - Returns the result or error (formatted using shared error utilities) from
//     the execution.
// - Providing functions to register native commands during Mountain startup
//   (`register_native_command_internal`).
// - Implementing the actual logic for native commands (e.g.,

//   `handle_native_save_all`).
//
// Key Interactions:
// - Interacts heavily with `AppState` to access/modify the `command_registry`.
// - Called by `track::dispatch_sidecar_request` or `rpc.rs` for RPC methods.
// - Called by `track::dispatch_command` (indirectly via effect) for execution.
// - Uses `vine::send_request_to_sidecar` to proxy command execution.
// - Executes native command logic directly or via `ActionEffect`s using
//   `AppRuntime`.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,
	future::Future,
	pin::Pin,
	sync::{Arc, Mutex as StdMutex, MutexGuard},
};

use Land_Common::{command_effects, errors::CommonError, ui_effects, workspace_effects};
use log::{debug, error, info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime as TauriRuntime, State, Window};

use crate::{
	app_state::{AppState, CommandHandler},

	// Use the shared error utilities
	handlers::error_utils,

	runtime::AppRuntime,

	vine,
};

// --- Helper Functions ---

/// Helper to map Mutex lock poisoning errors to the handler's error string
/// format.
fn map_app_state_lock_error_to_str<T>(e:std::sync::PoisonError<MutexGuard<'_, T>>, context:&str) -> String {
	let common_err = CommonError::StateLock(format!("Failed to acquire lock on {}: {}", context, e));

	error_utils::map_common_error_to_rpc_string(common_err, context)
}

// --- Request Handlers (Called by Track dispatcher or rpc.rs for
// sidecar/frontend requests) ---

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
		.ok_or_else(|| error_utils::rpc_param_error_string("handle_register_command", "id", "string", None))?
		.to_string();

	// Keep: Essential registration action log
	info!("[Cmd Handler] Registering PROXY for '{}' from '{}'", command_id, sidecar_id);

	let app_state = app.state::<AppState>();

	let mut registry = app_state
		.command_registry
		.lock()
		.map_err(|e| map_app_state_lock_error_to_str(e, "command_registry"))?;

	// Keep: Warning for overwrite is useful
	if registry.contains_key(&command_id) {
		warn!(
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
	info!("[Cmd Handler] Command '{}' registered successfully in AppState.", command_id);

	// TODO: Register ownership if implemented

	Ok(Value::Null)
}

/// Handles the `commands_unregisterCommand` request from a sidecar shim.
/// Removes the command ID association for the originating sidecar (if owned).
pub async fn handle_unregister_command<R:TauriRuntime>(
	app:AppHandle<R>,

	// Used for logging, potentially for ownership check in future
	sidecar_id:String,

	params:Value,
) -> Result<Value, String> {
	let command_id_str = params
		.get("id")
		.and_then(|v| v.as_str())
		.ok_or_else(|| error_utils::rpc_param_error_string("handle_unregister_command", "id", "string", None))?;

	// Keep: Essential unregistration action log
	info!(
		"[Cmd Handler] Unregistering command '{}' from sidecar '{}'",
		command_id_str, sidecar_id
	);

	let app_state = app.state::<AppState>();

	let mut registry = app_state
		.command_registry
		.lock()
		.map_err(|e| map_app_state_lock_error_to_str(e, "command_registry"))?;

	// TODO: Implement ownership check before removing.
	// For now, simple removal:
	// Keep: Confirmation or warning log
	if registry.remove(command_id_str).is_some() {
		info!("[Cmd Handler] Command '{}' unregistered.", command_id_str);

		// TODO: Also remove from ownership map if implemented
	} else {
		warn!("[Cmd Handler] Command '{}' not found for unregistration.", command_id_str);
	}

	// Success, void operation even if not found
	Ok(Value::Null)
}

/// Handles the `commands_getCommands` request from a sidecar shim.
/// Returns a list of *all* registered command IDs (native and proxied).
pub async fn handle_get_commands<R:TauriRuntime>(
	app:AppHandle<R>,

	// Keep for signature consistency with other handlers
	_runtime:State<'_, Arc<AppRuntime>>,
) -> Result<Value, String> {
	// Reduced logging from previous trace! to avoid noise for frequent calls
	debug!("[Cmd Handler] Handling getCommands request");

	let app_state = app.state::<AppState>();

	let registry = app_state
		.command_registry
		.lock()
		.map_err(|e| map_app_state_lock_error_to_str(e, "command_registry"))?;

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
		.ok_or_else(|| error_utils::rpc_param_error_string("handle_execute_command", "id", "string", Some(0)))?
		.to_string();

	// Default to Null if "args" key is missing or not an array
	let args = params.get("args").cloned().unwrap_or(Value::Null);

	// Keep: Essential action log
	info!("[Cmd Handler] Handling executeCommand '{}'", command_id);

	trace!("[Cmd Handler] Args for {}: {:?}", command_id, args);

	let app_state = app.state::<AppState>();

	// Get handler info while holding lock briefly
	let handler_info = {
		let registry = app_state
			.command_registry
			.lock()
			.map_err(|e| map_app_state_lock_error_to_str(e, "command_registry"))?;

		registry.get(&command_id).cloned()
		// Lock released
	};

	match handler_info {
		Some(CommandHandler::Native(native_handler_fn)) => {
			// Keep: Distinguishes native execution path
			debug!("[Cmd Handler] Found Native handler for '{}'. Executing...", command_id);

			// Pass runtime.inner().clone() which is Arc<AppRuntime>
			native_handler_fn(app, window, runtime.inner().clone(), args).await
		},

		Some(CommandHandler::Proxied { sidecar_id, command_id: proxied_cmd_id }) => {
			// Keep: Distinguishes proxied execution path
			debug!(
				"[Cmd Handler] Found Proxied handler for '{}'. Routing to sidecar '{}'",
				proxied_cmd_id, sidecar_id
			);

			let request_payload = json!({ "id": proxied_cmd_id, "args": args });

			let sidecar_method = "commands_executeContributedCommand".to_string();

			// Use the more specific vine::send_request_to_sidecar
			// 30s timeout
			vine::send_request_to_sidecar(&sidecar_id, sidecar_method, request_payload, 30000)
				.await
				.map_err(|e| {
					error!(
						"[Cmd Handler] Vine error routing command '{}' to sidecar '{}': {}",
						command_id, sidecar_id, e
					);

					error_utils::rpc_error_string(
						format!("Failed to route command '{}' to sidecar '{}': {}", command_id, sidecar_id, e),
						Some("EIPC"),
					)
				})
		},

		None => {
			// Keep: Important error log
			error!("[Cmd Handler] Command '{}' not found in registry.", command_id);

			Err(error_utils::rpc_error_string(
				format!("Command '{}' not found.", command_id),
				Some("ENOCMD"),
			))
		},
	}
}

// --- Native Command Handler Implementations ---

/// Native handler for `workbench.action.files.saveAll`. Runs the effect.
pub fn handle_native_save_all<R:TauriRuntime>(
	_app:AppHandle<R>,

	_window:Window<R>,

	// Native handlers take Arc<AppRuntime> directly
	runtime:Arc<AppRuntime>,

	args:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		let include_untitled = args.get(0).and_then(|v| v.as_bool()).unwrap_or(true);

		info!("[Native Cmd] Executing saveAll (includeUntitled: {})", include_untitled);

		let effect = workspace_effects::save_all(include_untitled);

		// The save_all effect returns Vec<bool> indicating success of each save op
		runtime
			.run(effect)
			.await
			.map(|results_vec_bool| json!(results_vec_bool))
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "native_save_all"))
	})
}

/// Native handler for `mountain.action.showAbout`. Runs the effect.
pub fn handle_native_show_about<R:TauriRuntime>(
	// Use app_handle to get package_info for version
	app_handle:AppHandle<R>,

	_window:Window<R>,

	runtime:Arc<AppRuntime>,

	_args:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		info!("[Native Cmd] Executing showAbout");

		let version = app_handle.package_info().version.to_string();

		let message = format!("Land Editor (Mountain)\nVersion: {}\nMore info at our website.", version);

		// Assuming ui_effects::show_message takes options for buttons, etc.
		// For a simple about dialog, no specific options might be needed if handled by
		// frontend.
		let effect = ui_effects::show_message("info".to_string(), message, None);

		// show_message effect returns Option<String> (selected button if any)
		runtime
			.run(effect)
			.await
			.map(|opt_str_result| json!(opt_str_result))
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "native_show_about"))
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
		warn!(
			"[Cmd Registry Init] Warning: Native command ID '{}' collision during registration.",
			command_id
		);
	}

	// Keep: Log registration during init
	info!("[Cmd Registry Init] Registered native command: {}", command_id);

	registry.insert(command_id, CommandHandler::Native(handler));
}
