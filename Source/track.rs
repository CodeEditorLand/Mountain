// ---------------------------------------------------------------------------------------------
// Mountain Track - Command and Request Dispatcher (track.rs)
// --------------------------------------------------------------------------------------------
// Acts as the central routing hub for actions within Mountain, originating from
// both the Sky frontend (via Tauri `invoke`) and sidecar processes like Cocoon
// (via the Vine IPC layer). Its primary role is to translate these incoming
// commands and requests into abstract `ActionEffect`s (defined in
// `Land_Common`) or to route them to direct handler functions or RPC struct
// methods if an effect mapping is not appropriate or available.
//
// `ActionEffect`s are then dispatched to the `AppRuntime` for execution, which
// uses the `MountainEnvironment` to perform the actual work.
//
// Responsibilities:
// - Implementing the primary Tauri `#[command]` function (`dispatch_command`)
//   which serves as the entry point for all commands invoked from the Sky
//   frontend.
// - Providing the `dispatch_sidecar_request` function, which is called by the
//   `Vine` IPC layer when a request or notification is received from a sidecar.
// - Parsing command/method names and their associated arguments
//   (`serde_json::Value`).
// - Prioritizing direct handling for specific notifications from sidecars that
//   don't fit the request/response or effect pattern (e.g., terminal
//   environment variable updates, extension lifecycle notifications).
// - Mapping incoming command names (from Sky) and RPC method names (from
//   sidecars) to their corresponding `ActionEffect` constructors defined in
//   `Land_Common::effects`. This is the preferred way to handle operations that
//   involve state changes, I/O, or complex logic, promoting a clear separation
//   of concerns.
// - If a sidecar request cannot be mapped to an `ActionEffect` (e.g., it's a
//   method not covered by the effect system or a very simple query), it falls
//   back to:
//   - Invoking methods on specific RPC handler structs defined in `rpc.rs`
//     (e.g., `MainThreadCommandsHandler`, `MainThreadFileSystemApiHandler`).
//   - Calling direct handler functions in `handlers::*` submodules (less common
//     now for primary logic, but still used for some specific cases).
// - Invoking `AppRuntime::run(effect)` to execute `ActionEffect`s.
// - Formatting success responses (`Ok(Value)`) and error responses
//   (`Err(String)`) for the caller (Sky or Vine) using shared error utilities
//   (`handlers::error_utils`).
//
// Key Interactions:
// - `dispatch_command`: Called by Tauri when Sky uses
//   `invoke('dispatch_command', ...)`.
// - `dispatch_sidecar_request`: Called by `vine.rs` when processing messages
//   from sidecars.
// - Uses command name constants from `Land_Echo` for frontend command mapping.
// - Uses RPC method names from VS Code's `extHost.protocol.ts` as the contract
//   for sidecar request mapping.
// - Creates `ActionEffect` instances from various modules in
//   `Land_Common::effects` (e.g., `fs_effects`, `config_effects`,

//   `document_effects`).
// - Uses `AppRuntime` (obtained via `State<'_, Arc<AppRuntime>>`) to execute
//   effects.
// - If falling back from effects, calls methods on handler structs in `rpc.rs`
//   or functions in `handlers::*`.
// - Utilizes `handlers::error_utils` for consistent JSON-RPC error string
//   formatting.
// --------------------------------------------------------------------------------------------
use std::{path::PathBuf, sync::Arc};

// Import effect constructors and DTOs from Land_Common
use Land_Common::{
	command_effects,

	// For effect creation
	config_effects::{self, ConfigurationTarget, IConfigurationOverrides},

	diagnostics_effects,

	documents_effects,

	// The core ActionEffect type
	effect::ActionEffect,

	// For error types returned by effects
	errors::CommonError,

	// fs_effects for FS actions, FsReader for generic effect wrapper
	fs_effects::{self, FsReader},

	ipc_effects,

	language_feature_effects::{self, ProviderType as CommonLangProviderType},

	output_effects,

	secrets_effects,

	storage_effects,

	ui_effects,

	workspace_effects,
};
// Constants for frontend command names (e.g., Land_Echo::REQUEST_READ_FILE)
use Land_Echo;
// Logging
use log::{debug, error, info, trace, warn};
// `serde::Deserialize` might be used if parsing complex DTOs from `args` directly in Track.
// use serde::Deserialize;
use serde_json::{Value, json};
use tauri::{AppHandle, Runtime as TauriRuntime, State, Window, command};
// For handling URIs in effect parameters
use url::Url;

use crate::{
	// `AppState` is not directly used by Track, but context for handlers/effects it calls.
	// app_state::AppState,

	// Access to direct handler functions (e.g., handlers::extension_status)
	handlers,

	// Centralized error utilities for RPC responses
	handlers::error_utils,

	// Access to RPC handler structs (e.g., rpc::MainThreadCommandsHandler)
	rpc,

	runtime::AppRuntime, /* For running ActionEffects
	                      * `vine` is not directly called by Track; Vine calls Track. */
};

// --- Error Handling Abstraction ---
// These functions now directly use `handlers::error_utils` for consistency.

/// Creates a JSON-RPC error string for parameter parsing failures.
///
/// Delegates to `error_utils::rpc_param_error_string`.
fn create_parameter_parse_error_string(
	method_name:&str,

	param_name:&str,

	expected_type:&str,

	index:Option<usize>,
) -> String {
	error_utils::rpc_param_error_string(method_name, param_name, expected_type, index)
}

/// Maps a `CommonError` (from effect execution) to a JSON-RPC error string.
/// Delegates to `error_utils::map_common_error_to_rpc_string`.
fn map_common_error_to_rpc_error_string(e:CommonError, operation_context:&str) -> String {
	error_utils::map_common_error_to_rpc_string(e, operation_context)
}

// --- Frontend Command Dispatcher (`#[tauri::command]`) ---

/// Main entry point for commands invoked from the Sky frontend via Tauri's
/// `invoke` system.
///
/// This function attempts to map the `command` string (typically a constant
/// from `Land_Echo`) to an `ActionEffect`. If successful, the effect is run
/// using the `AppRuntime`.
///
/// # Arguments
/// * `app_handle` - The Tauri `AppHandle`.
/// * `window` - The Tauri `Window` context.
/// * `runtime` - Managed `Arc<AppRuntime>` for executing effects.
/// * `command` - The string identifier of the command to dispatch (e.g.,
///   `Land_Echo::REQUEST_READ_FILE`).
/// * `args` - A `serde_json::Value` containing the arguments for the command,
///   typically an object.
///
/// # Returns
/// * `Result<Value, String>`:
///   - `Ok(Value)` with the result of the command/effect execution.
///   - `Err(String)` containing a JSON-RPC formatted error string if effect
///     creation or execution fails.
// Tauri attribute to expose this function as a command callable from frontend
#[command]
pub async fn dispatch_command<R:TauriRuntime>(
	app_handle:AppHandle<R>,

	window:Window<R>,

	// Access managed AppRuntime
	runtime:State<'_, Arc<AppRuntime>>,

	// Command ID string from frontend
	command:String,

	// Arguments for the command, typically a JSON object
	args:Value,
) -> Result<Value, String> {
	info!("[Track FrontendCmd Dispatch] Received command: '{}'", command);

	trace!("[Track FrontendCmd Dispatch] Args: {:?}", args);

	match create_effect_for_frontend_command(&app_handle, &window, &command, args) {
		Ok(effect_to_run) => {
			// Successfully created an effect, now run it.
			runtime.run(effect_to_run).await.map_err(|common_err_from_effect| {
				// Effect execution resulted in a CommonError.
				error!(
					"[Track FrontendCmd Dispatch] Error running effect for command '{}': {}",
					command, common_err_from_effect
				);

				// Map CommonError to a JSON-RPC error string for Sky.
				map_common_error_to_rpc_error_string(
					common_err_from_effect,
					&format!("frontend_command_execution_{}", command),
				)
			})
		},

		Err(effect_creation_err_str) => {
			// Effect creation failed (e.g., bad parameters).
			// `effect_creation_err_str` is already a JSON-RPC formatted error string.
			error!(
				"[Track FrontendCmd Dispatch] Error creating effect for command '{}': {}",
				command, effect_creation_err_str
			);

			Err(effect_creation_err_str)
		},
	}
}

// --- Sidecar Request/Notification Dispatcher (Called by Vine) ---

/// Dispatches requests and notifications received from a sidecar process (e.g.,
/// Cocoon) via the Vine IPC layer.
///
/// This function tries to:
/// 1. Directly handle specific notifications that don't fit request/response or
///    effect patterns (e.g., terminal env changes, extension lifecycle).
/// 2. Map the incoming RPC method name to an `ActionEffect` and execute it.
/// 3. If no effect mapping, fall back to invoking methods on RPC handler
///    structs in `rpc.rs` or direct handler functions in `handlers::*`.
///
/// # Arguments
/// * `app_handle` - The Tauri `AppHandle`.
/// * `window` - The Tauri `Window` context.
/// * `runtime` - Managed `Arc<AppRuntime>`.
/// * `sidecar_id` - Identifier of the sidecar sending the request/notification.
/// * `request_message_val` - A `serde_json::Value` representing the Vine
///   message, expected to be an object `{ "method": string, "params": Value }`.
///
/// # Returns
/// * `Result<Value, String>`:
///   - `Ok(Value)` for successful RPC request responses.
///   - `Ok(Value::Null)` for notifications (as they don't expect a return
///     value).
///   - `Err(String)` (JSON-RPC error string) for errors.
pub async fn dispatch_sidecar_request<R:TauriRuntime>(
	app_handle:AppHandle<R>,

	// Main window context
	window:Window<R>,

	// Managed AppRuntime
	runtime:State<'_, Arc<AppRuntime>>,

	// ID of the originating sidecar
	sidecar_id:String,

	// Raw Vine message: { method, params }
	request_message_val:Value,
) -> Result<Value, String> {
	// Default to empty if "method" is missing/not string
	let rpc_method_name = request_message_val.get("method").and_then(Value::as_str).unwrap_or("");

	// Params can be anything, default to Null
	let rpc_params_val = request_message_val.get("params").cloned().unwrap_or(Value::Null);

	info!(
		"[Track SidecarReq Dispatch] From sidecar '{}': Method='{}'",
		sidecar_id, rpc_method_name
	);

	trace!(
		"[Track SidecarReq Dispatch] Params (type='{:?}'): {}...",
		rpc_params_val.kind(),
		rpc_params_val
			.to_string()
			.chars()
			 // Log a sample of params
			.take(100)
			.collect::<String>()
	);

	// --- 1. Prioritize Direct Handling for Specific Notifications ---
	// Some notifications are better handled directly without going through effect
	// or full RPC machinery.
	if rpc_method_name.starts_with("terminal_") && rpc_method_name != "$createTerminal" {
		// These are environment variable change notifications from Cocoon's terminal
		// env collection.
		debug!(
			"[Track SidecarReq Dispatch] Routing terminal environment notification '{}' directly to handler.",
			rpc_method_name
		);

		return match rpc_method_name {
			"terminal_setEnvironmentVariable" => {
				handlers::terminal::handle_set_environment_variable_contribution(app_handle, rpc_params_val).await
			},

			"terminal_deleteEnvironmentVariable" => {
				handlers::terminal::handle_delete_environment_variable_contribution(app_handle, rpc_params_val).await
			},

			"terminal_clearEnvironmentVariableCollection" => {
				handlers::terminal::handle_clear_environment_variable_collection_contributions(
					app_handle,
					rpc_params_val,
				)
				.await
			},

			_ => {
				warn!(
					"[Track SidecarReq Dispatch] Received unknown direct terminal notification: {}",
					rpc_method_name
				);

				Err(error_utils::rpc_error_string(
					format!("Unknown direct terminal notification: {}", rpc_method_name),
					Some("ENOSYS_TERM_NOTIF_UNKNOWN"),
				))
			},
		};
	}

	// Handle extension lifecycle notifications directly.
	match rpc_method_name {
		"$log" | "$logExtensionHostActivation" | "$logExtensionHostRequest" => {
			// These are logging calls from Cocoon's general logger or specific log points.
			let rpc_log_handler = rpc::MainThreadLogHandler { app_handle, runtime:runtime.inner().clone() };

			return rpc_log_handler.log(rpc_params_val).await;
		},

		"$onWillActivateExtension"
		| "$onDidActivateExtension"
		| "$onExtensionActivationError"
		| "$onExtensionRuntimeError" => {
			// These are notifications about extension activation status.
			// `rpc_params_val` is expected to be an array by
			// `handle_extension_host_status_notification`.
			let params_as_array = rpc_params_val.as_array().cloned().unwrap_or_default();

			return handlers::extension_status::handle_extension_host_status_notification(
				app_handle,
				rpc_method_name,
				Value::Array(params_as_array),
			)
			.await;
		},

		_ => { /* Not a direct notification, continue to effect/RPC logic. */ },
	}

	// --- 2. Attempt Effect Creation for RPC Requests (methods starting with '$')
	// --- `rpc_params_val` is usually an array for methods from
	// `extHost.protocol.ts`.
	let params_array_for_effects = rpc_params_val
		.as_array()
		.cloned()
		 // Wrap non-array params in a vec
		.unwrap_or_else(|| vec![rpc_params_val.clone()]);

	match create_effect_for_sidecar_request(
		&sidecar_id,
		rpc_method_name,
		// Clone for potential fallback
		params_array_for_effects.clone(),
	) {
		Ok(effect_to_run) => {
			debug!(
				"[Track SidecarReq Dispatch] Successfully mapped RPC method '{}' to an ActionEffect. Running effect...",
				rpc_method_name
			);

			return runtime.run(effect_to_run).await.map_err(|common_err| {
				error!(
					"[Track SidecarReq Dispatch] Error running effect for RPC method '{}': {}",
					rpc_method_name, common_err
				);

				map_common_error_to_rpc_error_string(
					common_err,
					&format!("sidecar_effect_execution_{}", rpc_method_name),
				)
			});
		},

		Err(EffectCreationError::NoEffectMapping) => {
			// No direct effect mapping found, proceed to RPC handler fallback.
			debug!(
				"[Track SidecarReq Dispatch] No direct ActionEffect mapping for RPC method '{}'. Attempting fallback \
				 to RPC/direct handlers.",
				rpc_method_name
			);
		},

		Err(EffectCreationError::ParamParseError(param_err_str)) => {
			// Parameter parsing failed during effect creation. `param_err_str` is already
			// JSON-RPC formatted.
			error!(
				"[Track SidecarReq Dispatch] Parameter parsing error while creating effect for RPC method '{}': {}",
				rpc_method_name, param_err_str
			);

			return Err(param_err_str);
		},
	}

	// --- 3. Fallback to Direct RPC Handler Methods or Specific `handlers::*`
	// functions ---
	debug!(
		"[Track SidecarReq Dispatch] Attempting direct RPC handler fallback for method: '{}'",
		rpc_method_name
	);

	// Get Arc<AppRuntime> for handlers
	let rpc_handler_runtime_clone = runtime.inner().clone();

	// Match known RPC methods to their handlers.
	// Note: `rpc_params_val` is used here, not `params_array_for_effects`, as RPC
	// handlers       expect the original params structure (often an array, but
	// sometimes an object).
	match rpc_method_name {
		// Commands
		"$executeCommand" | "$getCommands" | "$registerCommand" | "$unregisterCommand" => {
			let handler = rpc::MainThreadCommandsHandler { app_handle, runtime:rpc_handler_runtime_clone };

			match rpc_method_name {
				"$executeCommand" => handler.executeCommand(rpc_params_val).await,

				"$getCommands" => handler.getCommands(rpc_params_val).await,

				"$registerCommand" => handler.registerCommand(rpc_params_val).await,

				"$unregisterCommand" => handler.unregisterCommand(rpc_params_val).await,

				// Covered by outer match
				_ => unreachable!(),
			}
		},

		// Workspace
		"$resolveWorkspaceFolder" => {
			let handler = rpc::MainThreadWorkspaceHandler { app_handle, runtime:rpc_handler_runtime_clone };

			handler.resolveWorkspaceFolder(rpc_params_val).await
		},

		"$findFiles" => {
			// This one calls `handlers::workspace` directly.
			handlers::workspace::handle_find_files(app_handle, rpc_params_val).await
		},

		// Language Feature Registrations (should be effects, this is a deep fallback)
		_ if rpc_method_name.starts_with("$register") && rpc_method_name.contains("Provider") => {
			warn!(
				"[Track SidecarReq Dispatch] Language feature registration RPC '{}' fell back to RPC handler (should \
				 be an ActionEffect). This indicates a missing mapping in `create_effect_for_sidecar_request`.",
				rpc_method_name
			);

			// Use the catch-all from rpc.rs for this unlikely case.
			let handler = rpc::MainThreadLanguageFeaturesHandler { app_handle, runtime:rpc_handler_runtime_clone };

			handler.CatchAllRegisterProvider(rpc_method_name, rpc_params_val).await
		},

		// UI Messages & Dialogs (these use effects internally via rpc.rs handlers)
		"$showMessage" => {
			let handler = rpc::MainThreadMessageHandler { app_handle, runtime:rpc_handler_runtime_clone };

			handler.showMessage(rpc_params_val).await
		},

		"$showOpenDialog" | "$showSaveDialog" => {
			let handler = rpc::MainThreadDialogsHandler { app_handle, runtime:rpc_handler_runtime_clone };

			match rpc_method_name {
				"$showOpenDialog" => handler.showOpenDialog(rpc_params_val).await,

				"$showSaveDialog" => handler.showSaveDialog(rpc_params_val).await,

				_ => unreachable!(),
			}
		},

		// Window
		"$focusWindow" => {
			let handler = rpc::MainThreadWindowHandler { app_handle, runtime:rpc_handler_runtime_clone };

			handler.focusWindow(rpc_params_val).await
		},

		// Status Bar
		"$setEntry" | "$disposeEntry" if rpc_method_name == "$setEntry" || rpc_method_name == "$disposeEntry" => {
			let handler = rpc::MainThreadStatusBarHandler { app_handle, runtime:rpc_handler_runtime_clone };

			match rpc_method_name {
				// Expects DTO directly
				"$setEntry" => handler.setEntry(rpc_params_val).await,

				// Expects [id]
				"$disposeEntry" => handler.disposeEntry(rpc_params_val).await,

				_ => unreachable!(),
			}
		},

		// Filesystem API (vscode.workspace.fs)
		"$stat" | "$readDirectory" | "$readFile" | "$writeFile" | "$createDirectory" | "$delete" | "$rename"
		| "$copy" => {
			let fs_api_handler = rpc::MainThreadFileSystemApiHandler { app_handle, runtime:rpc_handler_runtime_clone };

			// These methods in `MainThreadFileSystemApiHandler` expect `rpc_params_val`
			// (the array).
			match rpc_method_name {
				"$stat" => fs_api_handler.stat(rpc_params_val).await,

				"$readDirectory" => fs_api_handler.read_directory(rpc_params_val).await,

				"$readFile" => fs_api_handler.read_file(rpc_params_val).await,

				"$writeFile" => fs_api_handler.write_file(rpc_params_val).await,

				"$createDirectory" => fs_api_handler.create_directory(rpc_params_val).await,

				"$delete" => fs_api_handler.delete(rpc_params_val).await,

				"$rename" => fs_api_handler.rename(rpc_params_val).await,

				"$copy" => fs_api_handler.copy(rpc_params_val).await,

				_ => unreachable!(),
			}
		},

		// Document Operations (direct handlers)
		"$tryOpenDocument" => handlers::documents::handle_try_open_document(app_handle, rpc_params_val).await,

		"$tryCreateDocument" => handlers::documents::handle_try_create_document(app_handle, rpc_params_val).await,

		"$trySaveDocument" => {
			let uri_dto_val = rpc_params_val.as_array().and_then(|a| a.get(0)).cloned().ok_or_else(|| {
				create_parameter_parse_error_string(
					rpc_method_name,
					"uriComponents (args[0])",
					"Value::Object",
					Some(0),
				)
			})?;

			handlers::documents::handle_try_save_document(app_handle, uri_dto_val).await
		},

		"$trySaveDocumentAs" => {
			let original_uri_dto_val = rpc_params_val.as_array().and_then(|a| a.get(0)).cloned().ok_or_else(|| {
				create_parameter_parse_error_string(
					rpc_method_name,
					"originalUriComponents (args[0])",
					"Value::Object",
					Some(0),
				)
			})?;

			handlers::documents::handle_try_save_document_as(app_handle, original_uri_dto_val).await
		},

		"$saveAll" => {
			let include_untitled_bool = rpc_params_val
				.as_array()
				.and_then(|a| a.get(0))
				.and_then(Value::as_bool)
				.unwrap_or(true);

			handlers::documents::handle_save_all(app_handle, include_untitled_bool).await
		},

		// Output Channels (direct handlers, some might have been effects if params were simpler)
		// Check using `is_output_method_fallback_candidate` to disambiguate from other uses of these common method
		// names.
		_ if is_output_method_fallback_candidate(rpc_method_name) => {
			match rpc_method_name {
				"$register" => handlers::output::handle_register_output_channel(app_handle, rpc_params_val).await,

				"$append" => handlers::output::handle_append_to_output_channel(app_handle, rpc_params_val).await,

				"$replace" => handlers::output::handle_replace_output_channel_content(app_handle, rpc_params_val).await,

				"$reveal" => handlers::output::handle_reveal_output_channel(app_handle, rpc_params_val).await,

				"$close" => handlers::output::handle_close_output_channel_view(app_handle, rpc_params_val).await,

				// Note: $clear and $dispose for output channels are handled below with param type checks.
				_ => {
					error!(
						"[Track SidecarReq Dispatch] Unhandled output method in fallback candidate check: '{}'",
						rpc_method_name
					);

					Err(error_utils::rpc_error_string(
						format!("Output method '{}' not fully routed in fallback.", rpc_method_name),
						Some("ENOSYS_OUT_FALLBACK_ROUTE"),
					))
				},
			}
		},

		// Diagnostics (direct handlers)
		// Note: `$clear` for diagnostics is an effect, so it's handled by `create_effect_for_sidecar_request`.
		"$changeMany" => handlers::diagnostics::handle_change_many(app_handle, rpc_params_val).await,

		"$getDiagnostics" => handlers::diagnostics::handle_get_diagnostics(app_handle, rpc_params_val).await,

		// Terminals (via RPC struct that calls specific handlers in `handlers::terminal`)
		"$createTerminal" | "$show" | "$hide" | "$sendText" => {
			let terminal_rpc_handler =
				rpc::MainThreadTerminalServiceHandler { app_handle, runtime:rpc_handler_runtime_clone };

			match rpc_method_name {
				// Expects options
				"$createTerminal" => terminal_rpc_handler.createTerminal(rpc_params_val).await,

				// object
				// Expects [id, preserveFocus?]
				"$show" => terminal_rpc_handler.show(rpc_params_val).await,

				// Expects [id]
				"$hide" => terminal_rpc_handler.hide(rpc_params_val).await,

				// Expects [id, text]
				"$sendText" => terminal_rpc_handler.sendText(rpc_params_val).await,

				_ => unreachable!(),
			}
		},

		// Disambiguation for $dispose and $clear based on parameter types (heuristic).
		"$dispose" if rpc_params_val.as_array().and_then(|a| a.get(0)?.as_u64()).is_some() => {
			info!("[Track SidecarReq Dispatch] Assuming '$dispose' with u64 param is for Terminal (fallback).");

			let terminal_rpc_handler =
				rpc::MainThreadTerminalServiceHandler { app_handle, runtime:rpc_handler_runtime_clone };

			// Expects [id: u64]
			terminal_rpc_handler.dispose(rpc_params_val).await
		},

		"$dispose" if rpc_params_val.as_array().and_then(|a| a.get(0)?.as_str()).is_some() => {
			info!(
				"[Track SidecarReq Dispatch] Assuming '$dispose' with string param is for Output Channel (fallback)."
			);

			// Expects [id: string]
			handlers::output::handle_dispose_output_channel(app_handle, rpc_params_val).await
		},

		"$clear" if rpc_params_val.as_array().and_then(|a| a.get(0)?.as_str()).is_some() => {
			// Note: $clear for Diagnostics owner is an effect. This handles $clear for
			// OutputChannel.
			info!("[Track SidecarReq Dispatch] Assuming '$clear' with string param is for Output Channel (fallback).");

			// Expects [id: string]
			handlers::output::handle_clear_output_channel(app_handle, rpc_params_val).await
		},

		// Default: Method not found in any dispatch path.
		_ => {
			error!(
				"[Track SidecarReq Dispatch] Unhandled RPC method '{}' from sidecar '{}'. No effect mapping AND no \
				 explicit RPC/direct handler found after fallback.",
				rpc_method_name, sidecar_id
			);

			Err(error_utils::rpc_error_string(
				format!(
					"RPC method '{}' is not implemented or mapped in the Track dispatcher.",
					rpc_method_name
				),
				Some("ENOSYS_TRACK_UNHANDLED"),
			))
		},
	}
}

/// Helper to check if a method name is a candidate for output channel fallback
/// logic. This is used to disambiguate common method names like `$register`,
///
///
///
///
/// `$dispose`.
fn is_output_method_fallback_candidate(method_name:&str) -> bool {
	matches!(method_name, "$register" | "$append" | "$replace" | "$reveal" | "$close")
}

/// Represents errors that can occur during the creation of an `ActionEffect`.
enum EffectCreationError {
	// Indicates no effect is defined for the given command/method.
	NoEffectMapping,

	// String is already a JSON-RPC error string from `create_parameter_parse_error_string`.
	ParamParseError(String),
}

// --- Effect Creation Logic ---

/// Creates an `ActionEffect` for commands originating from the Sky frontend.
///
/// # Arguments
/// * `_app_handle` - Unused, kept for signature consistency.
/// * `_window` - Unused, kept for signature consistency.
/// * `command_id_str` - The command ID string (from `Land_Echo` constants).
/// * `args_val` - `serde_json::Value` containing arguments for the command.
///
/// # Returns
/// * `Ok(ActionEffect)` if successful.
/// * `Err(String)` (JSON-RPC error string) if command is unknown or args are
///   invalid.
fn create_effect_for_frontend_command<R:TauriRuntime>(
	// Currently unused, but available if needed for context.
	_app_handle:&AppHandle<R>,

	// Currently unused.
	_window:&Window<R>,

	command_id_str:&str,

	// Expects args to be a JSON object from Sky
	args_val:Value,
) -> Result<ActionEffect<Arc<AppRuntime>, CommonError, Value>, String> {
	// Helper for creating parameter error strings specifically for frontend command
	// arg parsing.
	let frontend_param_err_fn = |param_name:&str, expected_type:&str| -> String {
		// No index for object props
		create_parameter_parse_error_string(command_id_str, param_name, expected_type, None)
	};

	// Helpers to get typed arguments from the `args_val` JSON object.
	let get_string_arg_from_obj = |key:&str| {
		args_val
			.get(key)
			.and_then(Value::as_str)
			.map(String::from)
			.ok_or_else(|| frontend_param_err_fn(key, "string"))
	};

	let get_path_buf_arg_from_obj = |key:&str| get_string_arg_from_obj(key).map(PathBuf::from);

	let get_i64_arg_from_obj = |key:&str| {
		args_val
			.get(key)
			.and_then(Value::as_i64)
			.ok_or_else(|| frontend_param_err_fn(key, "i64 number"))
	};

	let get_bool_arg_from_obj =
		|key:&str, default_val:bool| args_val.get(key).and_then(Value::as_bool).unwrap_or(default_val);

	let get_optional_value_arg_from_obj = |key:&str| args_val.get(key).cloned();

	let get_required_value_arg_from_obj = |key:&str| {
		args_val
			.get(key)
			.cloned()
			.ok_or_else(|| frontend_param_err_fn(key, "JSON value"))
	};

	trace!(
		"[Track CreateEffect Frontend] Command='{}', Args='{:?}'",
		command_id_str, args_val
	);

	match command_id_str {
		// Filesystem Effects
		Land_Echo::REQUEST_READ_FILE => {
			let file_path = get_path_buf_arg_from_obj("path")?;

			let read_file_effect = fs_effects::read_file(file_path);

			// Wrap the Vec<u8>-returning effect to return a base64-encoded JSON string,

			// as expected by Sky for file content.
			Ok(ActionEffect::new(Arc::new(move |env_accessor| {
				// Clone effect for closure
				let effect_clone = read_file_effect.clone();

				Box::pin(async move {
					let fs_reader_env:Arc<dyn FsReader + Send + Sync> = env_accessor.require();

					fs_reader_env
						.run_effect(effect_clone)
						.await
						 // Encode to base64 JSON string
						.map(|bytes_vec| json!(base64::encode(bytes_vec)))
				})
			})))
		},

		Land_Echo::REQUEST_WRITE_FILE => {
			Ok(fs_effects::write_file_string(
				get_path_buf_arg_from_obj("path")?,
				// Expects string content from Sky
				get_string_arg_from_obj("content")?,
				// Default to create=true
				get_bool_arg_from_obj("create", true),
				get_bool_arg_from_obj("overwrite", true), /* Default to overwrite=true (check if this is desired
				                                           * default) */
			))
		},

		Land_Echo::REQUEST_NEW_FILE => {
			Ok(fs_effects::create_file(
				get_path_buf_arg_from_obj("parentDir")?.join(get_string_arg_from_obj("name")?),
			))
		},

		Land_Echo::REQUEST_NEW_FOLDER => {
			Ok(fs_effects::create_directory(
				get_path_buf_arg_from_obj("parentDir")?.join(get_string_arg_from_obj("name")?),
				// `create_directory` effect is recursive
				true,
			))
		},

		Land_Echo::REQUEST_DELETE_PATH => {
			Ok(fs_effects::delete(
				get_path_buf_arg_from_obj("path")?,
				// Default to recursive for safety/convenience
				get_bool_arg_from_obj("recursive", true),
				// Default to permanent delete
				get_bool_arg_from_obj("useTrash", false),
			))
		},

		Land_Echo::REQUEST_RENAME_PATH => {
			let old_path_buf = get_path_buf_arg_from_obj("oldPath")?;

			let new_name_str = get_string_arg_from_obj("newName")?;

			let parent_dir_path = old_path_buf.parent().ok_or_else(|| {
				frontend_param_err_fn(
					"parent of oldPath",
					&format!("valid parent directory for '{}'", old_path_buf.display()),
				)
			})?;

			Ok(fs_effects::rename(
				old_path_buf,
				parent_dir_path.join(new_name_str),
				// Default to no overwrite
				get_bool_arg_from_obj("overwrite", false),
			))
		},

		Land_Echo::REQUEST_COPY_PATH => {
			Ok(fs_effects::copy(
				get_path_buf_arg_from_obj("sourcePath")?,
				get_path_buf_arg_from_obj("targetParentDir")?.join(get_string_arg_from_obj("newName")?),
				get_bool_arg_from_obj("overwrite", false),
			))
		},

		// Document Effects
		Land_Echo::REQUEST_SAVE_FILE => {
			Ok(documents_effects::try_save(
				Url::parse(&get_string_arg_from_obj("uri")?)
					.map_err(|e_url| frontend_param_err_fn("uri (parse error)", &e_url.to_string()))?,
			))
		},

		Land_Echo::REQUEST_SAVE_FILE_AS => {
			Ok(documents_effects::try_save_as(
				Url::parse(&get_string_arg_from_obj("originalUri")?)
					.map_err(|e_url| frontend_param_err_fn("originalUri (parse error)", &e_url.to_string()))?,
				// `newTargetUri` is optional; if None, UiProvider will prompt user.
				get_optional_value_arg_from_obj("newTargetUri")
				.and_then(|val| val.as_str().map(|s| Url::parse(s)))
				 // Option<Result<Url, E>> -> Result<Option<Url>, E>
				.transpose()
				.map_err(|e_url| {


					frontend_param_err_fn("newTargetUri (parse error)", &e_url.to_string())
				})?,
			))
		},

		Land_Echo::REQUEST_APPLY_EDITOR_CHANGES => {
			Ok(documents_effects::apply_changes(
				Url::parse(&get_string_arg_from_obj("uri")?)
					.map_err(|e_url| frontend_param_err_fn("uri (parse error)", &e_url.to_string()))?,
				get_i64_arg_from_obj("versionId")?,
				// Expects array of change DTOs
				get_required_value_arg_from_obj("changes")?,
				// Default to dirty after change
				get_bool_arg_from_obj("isDirty", true),
				get_bool_arg_from_obj("isUndoing", false),
				get_bool_arg_from_obj("isRedoing", false),
			))
		},

		Land_Echo::REQUEST_OPEN_FILE => {
			Ok(documents_effects::try_open(
				// Expects UriComponents DTO
				get_required_value_arg_from_obj("uriComponents")?,
				get_optional_value_arg_from_obj("languageId").and_then(|v| v.as_str().map(String::from)),
				get_optional_value_arg_from_obj("content").and_then(|v| v.as_str().map(String::from)),
			))
		},

		// IPC Effects (for frontend to call into sidecar via Mountain)
		Land_Echo::REQUEST_PROXY_EXT_HOST_CALL => {
			Ok(ipc_effects::proxy_call_to_sidecar(
				// TODO: Target sidecar ID should be configurable or part of args
				"cocoon-main".to_string(),
				// Expects { method, params } for sidecar
				get_required_value_arg_from_obj("callData")?,
			))
		},

		Land_Echo::REQUEST_ESTABLISH_HOST_CONNECTION => {
			// This might be for initializing Vine if not already done, or a health check.
			// For now, map to an IPC effect that Vine might handle.
			Ok(ipc_effects::establish_host_connection(
				// TODO: Target sidecar ID
				"cocoon-main".to_string(),
			))
		},

		// WebSocket related commands (Mist) - These might not be effects if handled directly.
		Land_Echo::REQUEST_WS_SEND | Land_Echo::REQUEST_WS_CONNECT => {
			// These are handled directly by `handlers::mist::handle_ws_send_command` or
			// similar, not typically as effects in the same way.
			// If they were to be effects, they'd need an `IpcProvider` or `MistProvider`.
			Err(error_utils::rpc_error_string(
				format!(
					"WebSocket command '{}' is not implemented via the general effect system. It may have a direct \
					 handler.",
					command_id_str
				),
				Some("ENOSYS_WS_EFFECT"),
			))
		},

		_ => {
			Err(error_utils::rpc_error_string(
				format!("Unknown frontend command ID '{}' received for effect creation.", command_id_str),
				Some("ENOSYS_CMD_UNKNOWN"),
			))
		},
	}
}

/// Attempts to create an `ActionEffect` for RPC requests originating from a
/// sidecar.
///
/// # Arguments
/// * `sidecar_id_str` - Identifier of the calling sidecar.
/// * `rpc_method_name` - The RPC method name (e.g., "$getConfiguration").
/// * `params_vec` - A `Vec<Value>` containing parameters for the RPC method.
///
/// # Returns
/// * `Ok(ActionEffect)` if a mapping exists and params are valid.
/// * `Err(EffectCreationError)` if no mapping, or if param parsing fails.
fn create_effect_for_sidecar_request(
	sidecar_id_str:&str,

	rpc_method_name:&str,

	// RPC methods typically take an array of parameters
	params_vec:Vec<Value>,
) -> Result<ActionEffect<Arc<AppRuntime>, CommonError, Value>, EffectCreationError> {
	// Helper for creating parameter parsing error for sidecar requests.
	let sidecar_param_err_fn = |param_name:&str, expected_type:&str, idx:usize| {
		EffectCreationError::ParamParseError(create_parameter_parse_error_string(
			rpc_method_name,
			param_name,
			expected_type,
			Some(idx),
		))
	};

	// Helpers to get typed parameters from the `params_vec` by index.
	let get_string_param_at_idx = |idx:usize, name_for_err:&str| {
		params_vec
			.get(idx)
			.and_then(Value::as_str)
			.map(String::from)
			.ok_or_else(|| sidecar_param_err_fn(name_for_err, "string", idx))
	};

	let get_u32_param_at_idx = |idx:usize, name_for_err:&str| {
		params_vec
			.get(idx)
			// Parse as u64 first for flexibility
			.and_then(Value::as_u64)
			// Then cast to u32
			.map(|v| v as u32)
			.ok_or_else(|| sidecar_param_err_fn(name_for_err, "u32 number", idx))
	};

	let get_optional_param_at_idx = |idx:usize| params_vec.get(idx).cloned();

	let get_required_param_at_idx = |idx:usize, name_for_err:&str| {
		params_vec
			.get(idx)
			.cloned()
			.ok_or_else(|| sidecar_param_err_fn(name_for_err, "JSON value", idx))
	};

	trace!(
		"[Track CreateEffect Sidecar] Method='{}', NumParams={}, Sidecar='{}'",
		rpc_method_name,
		params_vec.len(),
		sidecar_id_str
	);

	// Helper to wrap effects returning `u32` (like language provider handles)
	// into `Value`-returning effects for the dispatcher.
	let lang_feat_reg_effect_adapter = |effect_u32:ActionEffect<Arc<AppRuntime>, CommonError, u32>| -> Result<
		ActionEffect<Arc<AppRuntime>, CommonError, Value>,
		EffectCreationError,
	> {
		Ok(ActionEffect::new(Arc::new(move |env_accessor| {
			let effect_clone = effect_u32.clone();

			Box::pin(async move {
				// Convert u32 to Value::Number
				env_accessor.run(effect_clone).await.map(Value::from)
			})
		})))
	};

	// Helper for effects returning `()` (void).
	let lang_feat_void_effect_adapter = |effect_void:ActionEffect<Arc<AppRuntime>, CommonError, ()>| -> Result<
		ActionEffect<Arc<AppRuntime>, CommonError, Value>,
		EffectCreationError,
	> {
		Ok(ActionEffect::new(Arc::new(move |env_accessor| {
			let effect_clone = effect_void.clone();

			Box::pin(async move {
				// Convert () to Value::Null
				env_accessor.run(effect_clone).await.map(|_| Value::Null)
			})
		})))
	};

	match rpc_method_name {
		// --- Configuration Effects ---
		// Params: [section?: string, overrides?: IConfigurationOverridesDto, scopeToLanguage?: boolean]
		"$getConfiguration" => {
			Ok(config_effects::get_configuration(
				// section_key_opt
				params_vec.get(0).and_then(Value::as_str).map(String::from),
				// overrides_dto
				get_optional_param_at_idx(1).unwrap_or(Value::Null),
				// scope_to_language_opt
				params_vec.get(2).and_then(Value::as_bool),
			))
		},

		// Params: [target: number (ConfigTarget), key: string, value: any, overrides?: Dto, scopeToLang?: boolean]
		"$updateConfigurationOption" => {
			Ok(config_effects::update_configuration(
				get_string_param_at_idx(1, "key")?,
				get_required_param_at_idx(2, "value")?,
				// Cast to u32 for enum
				get_u32_param_at_idx(0, "target (ConfigurationTarget)")?,
				// overrides_dto
				get_optional_param_at_idx(3).unwrap_or(Value::Null),
				// scope_to_language_opt
				params_vec.get(4).and_then(Value::as_bool),
			))
		},

		// Params: [target: number, key: string, overrides?: Dto, scopeToLang?: boolean]
		"$removeConfigurationOption" => {
			Ok(config_effects::update_configuration(
				get_string_param_at_idx(1, "key")?,
				// Setting value to Null removes the key
				Value::Null,
				get_u32_param_at_idx(0, "target (ConfigurationTarget)")?,
				get_optional_param_at_idx(2).unwrap_or(Value::Null),
				params_vec.get(3).and_then(Value::as_bool),
			))
		},

		// Params: [key: string, overrides?: IConfigurationOverridesDto]
		"$inspect" => Ok(config_effects::inspect_configuration(get_string_param_at_idx(0, "key")?)),

		// --- Workspace Info Effects ---
		// Params: void
		"$getWorkspaceFolders" => Ok(workspace_effects::get_workspace_folders()),

		// Params: [options?: WorkspaceTrustRequestOptionsDto]
		"$requestWorkspaceTrust" => {
			Ok(workspace_effects::request_trust(
				// options_dto_opt
				get_optional_param_at_idx(0),
			))
		},

		// --- Storage & Secrets Effects ---
		// Cocoon shim sends params as an object: {scope, key} for getValue, {scope, key}, value for setValue
		// The effect constructors `storage_effects::*` expect this object as a single `Value` param.
		"$getValue" => {
			Ok(storage_effects::get_storage_item(get_required_param_at_idx(
				0,
				"storage target object {scope, key}",
			)?))
		},

		"$setValue" => {
			Ok(storage_effects::set_storage_item(
				get_required_param_at_idx(0, "storage target object {scope, key}")?,
				// Value is the second parameter
				get_required_param_at_idx(1, "value to set")?,
			))
		},

		// Params: [extensionId: string, key: string]
		"$getPassword" => {
			Ok(secrets_effects::get_secret(
				get_string_param_at_idx(0, "extensionId")?,
				get_string_param_at_idx(1, "key")?,
			))
		},

		// Params: [extensionId: string, key: string, value: string]
		"$setPassword" => {
			Ok(secrets_effects::store_secret(
				get_string_param_at_idx(0, "extensionId")?,
				get_string_param_at_idx(1, "key")?,
				get_string_param_at_idx(2, "value")?,
			))
		},

		// Params: [extensionId: string, key: string]
		"$deletePassword" => {
			Ok(secrets_effects::delete_secret(
				get_string_param_at_idx(0, "extensionId")?,
				get_string_param_at_idx(1, "key")?,
			))
		},

		// --- Language Features Registration Effects ---
		// Cocoon sends params for $register...Provider as:
		// [0: internal_cocoon_handle, 1: selector_dto, 2: options_dto?, 3: extensionId_dto?]
		// Our effect constructors typically take: (selector_dto, sidecar_id_str, options_dto_opt).
		// The `internal_cocoon_handle` (params_vec[0]) is ignored by these Mountain effect creators.
		// `sidecar_id_str` is available in this function's scope.
		"$registerHoverProvider" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_hover_provider(
				get_required_param_at_idx(1, "selector DTO")?,
				sidecar_id_str.to_string(),
				// options_dto_opt
				get_optional_param_at_idx(2),
			))
		},

		"$registerCompletionItemProvider" | "$registerCompletionsProvider" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_completion_provider(
				get_required_param_at_idx(1, "selector DTO")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		// ... (Similar mappings for all other $register...Provider methods) ...
		// Omitting for brevity, but assume they follow the pattern above, e.g.:

		// "$registerDefinitionProvider" | "$registerDefinitionSupport" =>
		// lang_feat_reg_effect_adapter(language_feature_effects::register_definition_provider(...)),

		// ... many more language feature registrations ...
		"$registerDefinitionProvider" | "$registerDefinitionSupport" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::Definition,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerDeclarationProvider" | "$registerDeclarationSupport" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::Declaration,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerImplementationProvider" | "$registerImplementationSupport" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::Implementation,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerTypeDefinitionProvider" | "$registerTypeDefinitionSupport" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::TypeDefinition,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerReferencesProvider" | "$registerReferencesSupport" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::References,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerDocumentHighlightProvider" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::DocumentHighlight,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerDocumentSymbolProvider" | "$registerDocumentSymbolSupport" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::DocumentSymbol,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		// WS Symbol has options at index 1, no selector normally
		"$registerWorkspaceSymbolProvider" | "$registerWorkspaceSymbolSupport" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::WorkspaceSymbol,
				Value::Null,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(1),
			))
		},

		"$registerCodeActionProvider" | "$registerCodeActionSupport" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::CodeAction,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerCodeLensProvider" | "$registerCodeLensSupport" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::CodeLens,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerDocumentFormattingEditProvider" | "$registerDocumentFormattingSupport" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::Formatting,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerDocumentRangeFormattingEditProvider" | "$registerDocumentRangeFormattingSupport" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::RangeFormatting,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerOnTypeFormattingEditProvider" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::OnTypeFormatting,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_required_param_at_idx(2, "onTypeFormattingOptionsDto")?,
			))
		},

		"$registerRenameProvider" | "$registerRenameSupport" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::Rename,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerDocumentLinkProvider" | "$registerDocumentLinkSupport" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::DocumentLink,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerDocumentColorProvider" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::Color,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerFoldingRangeProvider" | "$registerFoldingRangeSupport" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::FoldingRange,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerSelectionRangeProvider" | "$registerSelectionRangeSupport" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::SelectionRange,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerCallHierarchyProvider" | "$registerCallHierarchySupport" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::CallHierarchy,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerTypeHierarchyProvider" | "$registerTypeHierarchySupport" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::TypeHierarchy,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerLinkedEditingRangeProvider" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::LinkedEditingRange,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		"$registerInlayHintsProvider" => {
			lang_feat_reg_effect_adapter(language_feature_effects::register_provider(
				CommonLangProviderType::InlayHints,
				get_required_param_at_idx(1, "selector")?,
				sidecar_id_str.to_string(),
				get_optional_param_at_idx(2),
			))
		},

		// Unregister Language Feature Provider
		// Params: [mountain_provider_handle: u32]
		"$unregister" | "$unregisterProvider" => {
			lang_feat_void_effect_adapter(language_feature_effects::unregister_provider(get_u32_param_at_idx(
				0,
				"provider_handle (Mountain-generated)",
			)?))
		},

		// --- Diagnostics Effects ---
		// Params: [owner: string]
		// Note: `$changeMany` and `$getDiagnostics` are direct handlers, not effects here.
		// Check if this is for Diagnostics based on typical usage (single string arg for owner)
		"$clear"
			if rpc_method_name == "$clear"
				&& params_vec.len() == 1
				&& params_vec.get(0).map_or(false, Value::is_string) =>
		{
			Ok(diagnostics_effects::clear_owner_diagnostics(get_string_param_at_idx(
				0, "owner",
			)?))
		},

		// --- Default: No Effect Mapping ---
		// If the RPC method name doesn't match any known effect creation rule,

		// signal that it needs to be handled by RPC fallback or a direct handler.
		_ => Err(EffectCreationError::NoEffectMapping),
	}
}
