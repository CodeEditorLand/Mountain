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

/// Formats a `PoisonError` resulting from a failed Mutex lock on `AppState`
/// into a standardized RPC error string.
///
/// This function is used to convert internal locking errors into a format
/// suitable for returning to RPC callers, ensuring consistent error reporting.
///
/// # Arguments
/// * `e` - The `PoisonError` encountered.
/// * `context` - A string describing the context of the lock attempt (e.g.,
///
///   "command_registry").
///
/// # Returns
/// A `String` containing a JSON-formatted RPC error.
fn format_app_state_lock_error_for_rpc<T>(e:std::sync::PoisonError<MutexGuard<'_, T>>, context:&str) -> String {
	let common_err = CommonError::StateLock(format!("Failed to acquire lock on {}: {}", context, e));

	// Log the error internally before formatting for RPC
	error!("[LockError] Context: '{}', Error: {}", context, common_err);

	error_utils::map_common_error_to_rpc_string(common_err, context)
}

// --- Request Handlers (Called by Track dispatcher or rpc.rs for
// sidecar/frontend requests) ---

/// Handles the `commands_registerCommand` RPC request from a sidecar (e.g.,
///
/// Cocoon).
///
/// This function registers a *proxy* command handler in Mountain's central
/// command registry. When this proxied command is executed, Mountain will
/// forward the execution request back to the originating sidecar via Vine.
///
/// # Arguments
/// * `app` - The Tauri `AppHandle`, used to access `AppState`.
/// * `sidecar_id` - The identifier of the sidecar registering the command.
/// * `params` - A `serde_json::Value` expected to be an object containing an
///   `id` field with the command ID string.
///
/// # Returns
/// * `Ok(Value::Null)` on successful registration.
/// * `Err(String)` containing a JSON-RPC error string if parameter parsing
///   fails or if there's an issue accessing the command registry.
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
	info!(
		"[Cmd Handler] Registering PROXY command '{}' from sidecar '{}'",
		command_id, sidecar_id
	);

	let app_state = app.state::<AppState>();

	let mut registry = app_state
		.command_registry
		.lock()
		.map_err(|e| format_app_state_lock_error_for_rpc(e, "command_registry"))?;

	// Keep: Warning for overwrite is useful
	if registry.contains_key(&command_id) {
		warn!(
			"[Cmd Handler] Warning: Command ID '{}' is already registered. Overwriting.",
			command_id
		);

		// TODO: Add ownership tracking. If a command is overwritten, the
		// previous owner (if different) should ideally be notified or its
		// ownership revoked. This prevents a sidecar from inadvertently or
		// maliciously overwriting another's command.
	}

	// Insert the proxy handler, associating it with the originating sidecar.
	registry.insert(
		command_id.clone(),
		CommandHandler::Proxied { sidecar_id:sidecar_id.clone(), command_id:command_id.clone() },
	);

	// Keep: Confirmation of success
	info!(
		"[Cmd Handler] Command '{}' (proxy for sidecar '{}') registered successfully in AppState.",
		command_id, sidecar_id
	);

	// TODO: Register ownership of the command to `sidecar_id` in a separate
	// ownership map within AppState if/when ownership tracking is implemented.

	Ok(Value::Null)
}

/// Handles the `commands_unregisterCommand` RPC request from a sidecar.
///
/// This function attempts to remove a command from Mountain's registry.
/// Currently, it removes the command if it exists, regardless of which sidecar
/// registered it.
///
/// # Arguments
/// * `app` - The Tauri `AppHandle`.
/// * `sidecar_id` - The identifier of the sidecar requesting unregistration
///   (used for logging, future ownership checks).
/// * `params` - A `serde_json::Value` expected to be an object containing an
///   `id` field with the command ID string.
///
/// # Returns
/// * `Ok(Value::Null)` on success (even if the command was not found, as
///   unregistration is idempotent).
/// * `Err(String)` if parameter parsing fails or an internal error occurs.
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
		"[Cmd Handler] Unregistering command '{}' requested by sidecar '{}'",
		command_id_str, sidecar_id
	);

	let app_state = app.state::<AppState>();

	let mut registry = app_state
		.command_registry
		.lock()
		.map_err(|e| format_app_state_lock_error_for_rpc(e, "command_registry"))?;

	// TODO: Implement ownership check before removing.
	//       Only allow `sidecar_id` to unregister a command if it's the registered
	// owner. For now, simple removal:
	if registry.contains_key(command_id_str) {
		// TODO: Add ownership check here:
		// if is_command_owner(&registry, command_id_str, &sidecar_id) {

		//     registry.remove(command_id_str).is_some();

		//     info!("[Cmd Handler] Command '{}' unregistered by owner '{}'.",

		// TODO: Also remove from ownership map if
		// command_id_str, sidecar_id);

		// implemented } else {

		//     warn!("[Cmd Handler] Sidecar '{}' attempted to unregister command '{}'
		// but is not the owner. Denied.", sidecar_id, command_id_str);     //
		// return
		// Optionally return an error:
		// Err(error_utils::rpc_error_string(format!("Not authorized to unregister
		// command '{}'", command_id_str), Some("EAUTH"))); }

		if registry.remove(command_id_str).is_some() {
			info!("[Cmd Handler] Command '{}' unregistered.", command_id_str);

			// TODO: Also remove from ownership map if implemented
		}
	} else {
		warn!(
			"[Cmd Handler] Command '{}' not found for unregistration (requested by {}).",
			command_id_str, sidecar_id
		);
	}

	// Success, void operation even if not found or not owner (for now)
	Ok(Value::Null)
}

/// Handles the `commands_getCommands` RPC request from a sidecar.
///
/// Returns a flat list of all registered command IDs, including both native
/// Mountain commands and commands proxied from sidecars.
///
/// # Arguments
/// * `app` - The Tauri `AppHandle`.
/// * `_runtime` - The `AppRuntime` (currently unused, kept for signature
///   consistency).
///
/// # Returns
/// * `Ok(Value::Array(Vec<Value::String>))` containing the list of command IDs.
/// * `Err(String)` if an internal error occurs.
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
		.map_err(|e| format_app_state_lock_error_for_rpc(e, "command_registry"))?;

	let command_list:Vec<String> = registry.keys().cloned().collect();

	// Release lock
	drop(registry);

	Ok(json!(command_list))
}

/// Handles the `commands_executeCommand` request, which can originate from the
/// frontend (via Track) or a sidecar (via RPC).
///
/// It looks up the command ID in the registry.
/// - If it's a native Mountain command, the corresponding Rust function is
///   executed.
/// - If it's a proxied command (registered by a sidecar), an RPC request
///   (`commands_executeContributedCommand`) is sent via Vine to the owning
///   sidecar to perform the execution.
///
/// # Arguments
/// * `app` - The Tauri `AppHandle`.
/// * `window` - The `Window` context, potentially needed for native UI actions.
/// * `runtime` - The `AppRuntime` for executing native effects.
/// * `params` - A `serde_json::Value` expected to be an object like `{ "id":
///   "command.id", "args": [...] }`.
///
/// # Returns
/// * `Ok(Value)` with the result from the command's execution.
/// * `Err(String)` containing a JSON-RPC error string if the command is not
///   found, parameter parsing fails, or execution fails.
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

	// Default to Null if "args" key is missing or not an array.
	// Native handlers and sidecars should be prepared for Value::Null args.
	let args = params.get("args").cloned().unwrap_or(Value::Null);

	// Keep: Essential action log
	info!(
		"[Cmd Handler] Handling executeCommand '{}' with args (type: {:?})",
		command_id,
		args.kind()
	);

	trace!("[Cmd Handler] Full args for {}: {:?}", command_id, args);

	let app_state = app.state::<AppState>();

	// Get handler info while holding lock briefly
	let handler_info = {
		let registry = app_state
			.command_registry
			.lock()
			.map_err(|e| format_app_state_lock_error_for_rpc(e, "command_registry"))?;

		registry.get(&command_id).cloned()
		// Lock released
	};

	match handler_info {
		Some(CommandHandler::Native(native_handler_fn)) => {
			// Keep: Distinguishes native execution path
			debug!("[Cmd Handler] Found NATIVE handler for '{}'. Executing...", command_id);

			// Pass runtime.inner().clone() which is Arc<AppRuntime>
			native_handler_fn(app, window, runtime.inner().clone(), args).await
		},

		Some(CommandHandler::Proxied { sidecar_id, command_id: proxied_cmd_id }) => {
			// Keep: Distinguishes proxied execution path
			debug!(
				"[Cmd Handler] Found PROXIED handler for '{}'. Routing to sidecar '{}' (original ID in sidecar: '{}')",
				command_id, sidecar_id, proxied_cmd_id
			);

			// The payload to the sidecar should use the ID it originally registered.
			let request_payload = json!({ "id": proxied_cmd_id, "args": args });

			let sidecar_method = "commands_executeContributedCommand".to_string();

			// TODO: Consider making the timeout configurable or command-specific.
			// 30 seconds
			let timeout_ms = 30000;

			vine::send_request_to_sidecar(&sidecar_id, sidecar_method, request_payload, timeout_ms)
				.await
				.map_err(|e| {
					error!(
						"[Cmd Handler] Vine error routing command '{}' (proxied as '{}') to sidecar '{}': {}",
						command_id, proxied_cmd_id, sidecar_id, e
					);

					// Provide a structured error message back to the caller.
					error_utils::rpc_error_string(
						format!("Failed to route command '{}' to sidecar '{}': {}", command_id, sidecar_id, e),
						// Specific error code for proxy failure
						Some("EIPC_PROXY_COMMAND"),
					)
				})
		},

		None => {
			// Keep: Important error log
			error!("[Cmd Handler] Command '{}' not found in registry.", command_id);

			Err(error_utils::rpc_error_string(
				format!("Command '{}' not found.", command_id),
				// Error NO CoMmanD
				Some("ENOCMD"),
			))
		},
	}
}

// --- Native Command Handler Implementations ---

/// Native command handler for `workbench.action.files.saveAll`.
///
/// This function triggers the "save all" operation within the workspace,
///
/// potentially including untitled files. It runs the
/// `workspace_effects::save_all` effect.
///
/// # Arguments
/// * `_app` - The Tauri `AppHandle` (unused in this specific handler).
/// * `_window` - The `Window` context (unused).
/// * `runtime` - An `Arc<AppRuntime>` used to execute the effect.
/// * `args` - A `serde_json::Value`. Expected to be an array where the first
///   element (optional) is a boolean indicating whether to include untitled
///   files. Defaults to `true`.
///
/// # Returns
/// A pinned, boxed future that resolves to `Result<Value, String>`.
/// * `Ok(Value::Array(Vec<Value::Bool>))` where each boolean indicates the
///   success of saving an individual file.
/// * `Err(String)` if the effect execution fails.
pub fn handle_native_save_all<R:TauriRuntime>(
	_app:AppHandle<R>,

	_window:Window<R>,

	// Native handlers take Arc<AppRuntime> directly
	runtime:Arc<AppRuntime>,

	args:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		// Args for saveAll: [includeUntitled?: boolean]
		let include_untitled = args.get(0).and_then(|v| v.as_bool()).unwrap_or(true);

		info!(
			"[Native Cmd] Executing 'workbench.action.files.saveAll' (includeUntitled: {})",
			include_untitled
		);

		let effect = workspace_effects::save_all(include_untitled);

		// The save_all effect returns Vec<bool> indicating success of each save op
		runtime
			.run(effect)
			.await
			.map(|results_vec_bool| json!(results_vec_bool))
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "native_command_save_all"))
	})
}

/// Native command handler for `mountain.action.showAbout`.
///
/// Displays an "About" dialog containing information about the Land Editor,
///
/// including its version. It runs the `ui_effects::show_message` effect.
///
/// # Arguments
/// * `app_handle` - The Tauri `AppHandle`, used to get package information
///   (version).
/// * `_window` - The `Window` context (unused, as `ui_effects::show_message`
///   handles window association).
/// * `runtime` - An `Arc<AppRuntime>` for effect execution.
/// * `_args` - `serde_json::Value` (unused for this command).
///
/// # Returns
/// A pinned, boxed future that resolves to `Result<Value, String>`.
/// * `Ok(Value::Null)` or `Ok(Value::String)` representing the selected button
///   if the dialog had choices (typically null for simple about dialogs).
/// * `Err(String)` if the effect execution fails.
pub fn handle_native_show_about<R:TauriRuntime>(
	// Use app_handle to get package_info for version
	app_handle:AppHandle<R>,

	_window:Window<R>,

	runtime:Arc<AppRuntime>,

	_args:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		info!("[Native Cmd] Executing 'mountain.action.showAbout'");

		let version = app_handle.package_info().version.to_string();

		let app_name = &app_handle.package_info().name;

		// TODO: Consider making the "More info" URL configurable or part of
		// package_info.
		let message = format!("{} (Mountain)\nVersion: {}\n\nMore info at our website.", app_name, version);

		// For a simple about dialog, no specific options or buttons might be needed
		// if the frontend's ui_effects::show_message implementation handles a
		// default "OK" button for info dialogs.
		// If buttons were needed, options_val would be like:
		// let options_val = json!({ "items": [{"title": "OK"}] });

		// No custom buttons for simple about.
		let options_val = Value::Null;

		let effect = ui_effects::show_message(
			// Use the enum for clarity
			ui_effects::MessageSeverity::Info,
			message,
			options_val,
		);

		// show_message effect returns Option<String> (selected button title if any)
		runtime
			.run(effect)
			.await
			.map(|opt_str_result| json!(opt_str_result))
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "native_command_show_about"))
	})
}

// --- Native Command Registration Helper ---

/// Registers a native command handler function in the provided command
/// registry.
///
/// This helper is typically used during Mountain's startup phase (e.g., in
/// `app_state.rs` or `main.rs`) to populate the `command_registry` with
/// handlers for commands implemented directly in Rust.
///
/// # Type Parameters
/// * `R` - A type that implements `TauriRuntime` and is `'static`.
///
/// # Arguments
/// * `registry` - A mutable reference to the `HashMap` serving as the command
///   registry.
/// * `command_id` - The string ID of the command to register (e.g.,
///
///   "workbench.action.files.saveAll").
/// * `handler` - The function pointer to the native command handler. This
///   function must match the signature defined by `CommandHandler::Native`.
///
/// # Panics
/// This function does not panic directly, but operations on the `registry`
/// (like `insert`) could panic if memory allocation fails, though this is
/// highly unlikely in typical scenarios.
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
	// Keep: Warning for collisions is important during development.
	if registry.contains_key(&command_id) {
		warn!(
			"[Cmd Registry Init] Warning: Native command ID '{}' is already registered. Overwriting with new handler.",
			command_id
		);

		// TODO: Consider if overwriting native commands should be allowed or
		// should panic/error. For now, it overwrites, which might be useful
		// for hot-reloading in dev, but risky in prod.
	}

	// Keep: Log registration during init for audit and debugging.
	info!("[Cmd Registry Init] Registered native command: {}", command_id);

	registry.insert(command_id, CommandHandler::Native(handler));
}
