// ---------------------------------------------------------------------------------------------
// Mountain Command Handlers (handlers/commands.rs)
// --------------------------------------------------------------------------------------------
// Implements the core logic for managing and executing commands within
// Mountain, handling interactions originating from both the frontend (via
// Track) and sidecars (via RPC/Vine/Track). It maintains the central command
// registry.
//
// Responsibilities:
// - Managing the command registry in `AppState`.
// - Handling `$registerCommand`, `$unregisterCommand`, `$getCommands` RPCs from
//   sidecars.
// - Handling command execution requests (`handle_execute_command`):
//   - Detects and routes "delegating commands" (with `$ident`) back to Cocoon.
//   - Executes native Mountain commands.
//   - Proxies execution of sidecar-registered commands back to the owning
//     sidecar.
// - Providing native command implementations and registration helpers.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,
	future::Future,
	pin::Pin,
	sync::{Arc, MutexGuard as StdMutexGuard}, // Renamed to avoid conflict
};

use Land_Common::{command_effects, errors::CommonError, ipc_effects::ProxyTarget, ui_effects, workspace_effects}; /* Added ProxyTarget */
use log::{debug, error, info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime as TauriRuntime, State, Window};

use crate::{
	app_state::{AppState, CommandHandler},
	handlers::error_utils, // For formatting errors
	runtime::AppRuntime,
	vine, // For sending requests to sidecars
};

// --- Constants ---
/// Prefix for Cocoon's delegating command IDs (e.g., those returned with an
/// `$ident`).
const COCOON_DELEGATING_CMD_ID_PREFIX:&str = "_cocoon.executeContributedCommandWithCachedArgument";

// --- Helper Functions ---
fn format_app_state_lock_error_for_rpc<T>(e:std::sync::PoisonError<StdMutexGuard<'_, T>>, context:&str) -> String {
	let common_err = CommonError::StateLock(format!("[Cmds Handler LockErr] Failed lock on {}: {}", context, e));
	error!("{}", common_err);
	error_utils::map_common_error_to_rpc_string(common_err, context)
}

// --- Request Handlers (Called by Track dispatcher or rpc.rs) ---

pub async fn handle_register_command<R:TauriRuntime>(
	app:AppHandle<R>,
	sidecar_id:String,
	params:Value, // Expects { "id": "command.id" }
) -> Result<Value, String> {
	let command_id = params
		.get("id")
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("handle_register_command", "id", "string", None))?
		.to_string();

	info!(
		"[Cmd Handler] Registering PROXY command '{}' from sidecar '{}'",
		command_id, sidecar_id
	);
	let app_state = app.state::<AppState>();
	let mut registry = app_state
		.command_registry
		.lock()
		.map_err(|e| format_app_state_lock_error_for_rpc(e, "command_registry for register"))?;

	if registry.contains_key(&command_id) {
		warn!(
			"[Cmd Handler] Warning: Command ID '{}' already registered. Overwriting.",
			command_id
		);
		// TODO: Implement ownership tracking to prevent unauthorized
		// overwrites.
	}

	registry.insert(
		command_id.clone(),
		CommandHandler::Proxied { sidecar_id:sidecar_id.clone(), command_id:command_id.clone() },
	);
	info!(
		"[Cmd Handler] Command '{}' (proxy for sidecar '{}') registered.",
		command_id, sidecar_id
	);
	Ok(Value::Null)
}

pub async fn handle_unregister_command<R:TauriRuntime>(
	app:AppHandle<R>,
	sidecar_id:String, // Used for logging, future ownership checks
	params:Value,      // Expects { "id": "command.id" }
) -> Result<Value, String> {
	let command_id_str = params
		.get("id")
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("handle_unregister_command", "id", "string", None))?;

	info!(
		"[Cmd Handler] Unregistering command '{}' requested by sidecar '{}'",
		command_id_str, sidecar_id
	);
	let app_state = app.state::<AppState>();
	let mut registry = app_state
		.command_registry
		.lock()
		.map_err(|e| format_app_state_lock_error_for_rpc(e, "command_registry for unregister"))?;

	// TODO: Implement ownership check. Only owner should unregister.
	if registry.contains_key(command_id_str) {
		if registry.remove(command_id_str).is_some() {
			info!("[Cmd Handler] Command '{}' unregistered.", command_id_str);
		}
	} else {
		warn!(
			"[Cmd Handler] Command '{}' not found for unregistration (requested by {}).",
			command_id_str, sidecar_id
		);
	}
	Ok(Value::Null)
}

pub async fn handle_get_commands<R:TauriRuntime>(
	app:AppHandle<R>,
	_runtime:State<'_, Arc<AppRuntime>>, // Keep for signature consistency
) -> Result<Value, String> {
	debug!("[Cmd Handler] Handling getCommands request");
	let app_state = app.state::<AppState>();
	let registry = app_state
		.command_registry
		.lock()
		.map_err(|e| format_app_state_lock_error_for_rpc(e, "command_registry for getCommands"))?;
	let command_list:Vec<String> = registry.keys().cloned().collect();
	drop(registry);
	Ok(json!(command_list))
}

pub async fn handle_execute_command<R:TauriRuntime>(
	app:AppHandle<R>,
	window:Window<R>,                   // Window context for native UI actions
	runtime:State<'_, Arc<AppRuntime>>, // AppRuntime for native effects
	params:Value,                       // Expects { "id": "command.id", "args": Value (often array) }
) -> Result<Value, String> {
	let command_id_to_execute = params
		.get("id")
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("handle_execute_command", "params.id", "string", None))?
		.to_string();

	let original_args_val = params.get("args").cloned().unwrap_or(Value::Null);

	info!(
		"[Cmd Handler] Execute: ID='{}', ArgumentType='{:?}'",
		command_id_to_execute,
		original_args_val.kind()
	);
	trace!("[Cmd Handler] Full args for {}: {:?}", command_id_to_execute, original_args_val);

	// Check for Cocoon's delegating command pattern (e.g., command ID is a special
	// prefix, args contain $ident)
	if command_id_to_execute.starts_with(COCOON_DELEGATING_CMD_ID_PREFIX) {
		let ident_arg_array = original_args_val.as_array().ok_or_else(|| {
			error_utils::rpc_error_string(
				format!(
					"Delegating cmd '{}' expects args to be an array [$ident]",
					command_id_to_execute
				),
				Some("EBADARG_DELEGATE_CMD"),
			)
		})?;
		let ident_str = ident_arg_array.get(0).and_then(Value::as_str).ok_or_else(|| {
			error_utils::rpc_error_string(
				format!(
					"Delegating cmd '{}' received invalid $ident (not string or missing)",
					command_id_to_execute
				),
				Some("EBADARG_DELEGATE_IDENT"),
			)
		})?;

		info!(
			"[Cmd Handler] Detected Cocoon delegating command '{}' with $ident '{}'. Routing to \
			 Cocoon.$executeContributedCommand.",
			command_id_to_execute, ident_str
		);

		// Call Cocoon's $executeContributedCommand, passing $ident as commandId, and
		// empty args (as $ident implies cached args).
		let rpc_params_for_cocoon_delegate = json!([ident_str, []]); // commandId is $ident, args is empty array
		let rpc_method_on_cocoon =
			format!("{}$executeContributedCommand", ProxyTarget::ExtHostCommands.target_prefix());

		return vine::send_request_to_sidecar(
			"cocoon-main",
			rpc_method_on_cocoon,
			rpc_params_for_cocoon_delegate,
			30000,
		)
		.await
		.map_err(|e| {
			error_utils::rpc_error_string(
				format!("Failed to execute delegated command (ident '{}') on Cocoon: {}", ident_str, e),
				Some("EIPC_DELEGATE_EXEC_FAIL"),
			)
		});
	}

	// Standard command execution (native or proxied to sidecar)
	let app_state = app.state::<AppState>();
	let handler_info_opt = {
		let registry_guard = app_state
			.command_registry
			.lock()
			.map_err(|e| format_app_state_lock_error_for_rpc(e, "command_registry for execute"))?;
		registry_guard.get(&command_id_to_execute).cloned()
	};

	match handler_info_opt {
		Some(CommandHandler::Native(native_handler_fn)) => {
			debug!("[Cmd Handler] Executing NATIVE command '{}'.", command_id_to_execute);
			native_handler_fn(app, window, runtime.inner().clone(), original_args_val).await
		},
		Some(CommandHandler::Proxied { sidecar_id, command_id: proxied_cmd_id_in_sidecar }) => {
			debug!(
				"[Cmd Handler] Executing PROXIED command '{}' (as '{}') on sidecar '{}'.",
				command_id_to_execute, proxied_cmd_id_in_sidecar, sidecar_id
			);
			let rpc_params_for_cocoon = json!([proxied_cmd_id_in_sidecar, original_args_val]);
			let rpc_method_on_cocoon =
				format!("{}$executeContributedCommand", ProxyTarget::ExtHostCommands.target_prefix());

			vine::send_request_to_sidecar(&sidecar_id, rpc_method_on_cocoon, rpc_params_for_cocoon, 30000)
				.await
				.map_err(|e| {
					error_utils::rpc_error_string(
						format!(
							"Failed to execute proxied command '{}' on sidecar '{}': {}",
							command_id_to_execute, sidecar_id, e
						),
						Some("EIPC_PROXY_EXEC_FAIL"),
					)
				})
		},
		None => {
			error!(
				"[Cmd Handler] Command '{}' not found in registry for execution.",
				command_id_to_execute
			);
			Err(error_utils::rpc_error_string(
				format!("Command '{}' not found.", command_id_to_execute),
				Some("ENOCMD_EXEC"),
			))
		},
	}
}

// --- Native Command Handler Implementations ---
pub fn handle_native_save_all<R:TauriRuntime>(
	_app:AppHandle<R>,
	_window:Window<R>,
	runtime:Arc<AppRuntime>, // Takes Arc<AppRuntime> directly
	args:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		let include_untitled = args.get(0).and_then(Value::as_bool).unwrap_or(true);
		info!(
			"[Native Cmd] Executing 'workbench.action.files.saveAll' (includeUntitled: {})",
			include_untitled
		);
		let effect = workspace_effects::save_all_documents(include_untitled); // Corrected effect name
		runtime
			.run(effect)
			.await
			.map(|results_vec_bool| json!(results_vec_bool))
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "native_command_save_all"))
	})
}

pub fn handle_native_show_about<R:TauriRuntime>(
	app_handle:AppHandle<R>, // Use app_handle for package_info
	_window:Window<R>,
	runtime:Arc<AppRuntime>, // Takes Arc<AppRuntime> directly
	_args:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		info!("[Native Cmd] Executing 'mountain.action.showAbout'");
		let version = app_handle.package_info().version.to_string();
		let app_name = &app_handle.package_info().name;
		let message = format!("{} (Mountain)\nVersion: {}\n\nMore info at our website.", app_name, version);
		let options_val = Value::Null; // No custom buttons for simple about
		let effect = ui_effects::show_message(ui_effects::MessageSeverity::Info, message, options_val);
		runtime
			.run(effect)
			.await
			.map(|opt_str_result| json!(opt_str_result))
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "native_command_show_about"))
	})
}

// --- Native Command Registration Helper ---
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
	if registry.contains_key(&command_id) {
		warn!(
			"[Cmd Registry Init] Warning: Native command ID '{}' already registered. Overwriting.",
			command_id
		);
	}
	info!("[Cmd Registry Init] Registered native command: {}", command_id);
	registry.insert(command_id, CommandHandler::Native(handler));
}
