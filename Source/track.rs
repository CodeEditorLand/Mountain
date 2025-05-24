// ---------------------------------------------------------------------------------------------
// Mountain Track - Command Dispatcher (track.rs)
// --------------------------------------------------------------------------------------------
// Acts as the central routing hub for all actions within Mountain. It receives
// commands invoked from the frontend (Sky) via Tauri's `invoke` and
// requests/notifications proxied from sidecars (Cocoon) via the Vine IPC layer.
// Its primary role is to translate these incoming commands/requests into
// abstract `ActionEffect`s (defined in `Land_Common`) or route them to direct
// handler functions. Effects are then dispatched to the `AppRuntime` for
// execution.
//
// Responsibilities:
// - Implementing the Tauri `#[command]` function (`dispatch_command`) exposed
//   to the frontend.
// - Providing an internal dispatch function (`dispatch_sidecar_request`) called
//   by `Vine`.
// - Parsing incoming command names/methods and arguments (`Value`).
// - Prioritizing direct handling for specific notifications (e.g., `terminal_*`
//   env vars, extension lifecycle events).
// - Attempting to map command/method names to specific `ActionEffect`
//   constructors.
// - Falling back to direct handler function calls (`handlers::*`) or specific
//   RPC handler struct methods (`rpc::MainThread...Handler`) if effect creation
//   signals it or for specific methods designated for direct handling.
// - Invoking the `AppRuntime::run` method to execute created effects.
// - Handling errors during effect creation or execution and formatting them for
//   the caller.
//
// Key Interactions:
// - Called by Tauri (`dispatch_command`) and `Vine`
//   (`dispatch_sidecar_request`).
// - Uses definitions from `Land_Echo` (frontend commands) and
//   `extHost.protocol` (sidecar RPC methods).
// - Creates `ActionEffect` instances defined in `Land_Common`.
// - Interacts with `AppRuntime` (via Tauri `State`) to execute effects.
// - Calls functions defined in `handlers::*` modules for direct execution.
// - Calls methods on handler structs defined in `rpc::*` for some sidecar
//   requests.
// - Needs access to `AppHandle`, `Window`, `AppState`, `AppRuntime` (via
//   `State`).
// --------------------------------------------------------------------------------------------
use std::{path::PathBuf, sync::Arc};

use Land_Common::{
	command_effects,
	config_effects::{self, ConfigurationTarget, IConfigurationOverrides},
	diagnostics_effects,
	documents_effects,
	effect::ActionEffect,
	errors::CommonError,
	fs_effects,
	ipc_effects,
	language_feature_effects,
	output_effects,
	secrets_effects,
	storage_effects,
	ui_effects,
	workspace_effects,
};
// Frontend command constants
use Land_Echo;
use log::{debug, error, info, trace, warn};
use serde::Deserialize;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime as TauriRuntime, State, Window, command};
use url::Url;

use crate::{
	// Not directly used here, but handlers/effects use it
	app_state::AppState,

	// Direct handler implementations
	handlers,

	// RPC handler structs
	rpc,

	// Runtime for executing effects
	runtime::AppRuntime,

	// Might be needed if Track sends notifications directly
	vine,
};

// --- Frontend Command Dispatcher ---
#[command]
pub async fn dispatch_command<R:TauriRuntime>(
	// Renamed _app to app_handle for clarity, may be used
	app_handle:AppHandle<R>,

	// Renamed _window for clarity
	window:Window<R>,

	runtime:State<'_, Arc<AppRuntime>>,

	command:String,

	args:Value,
) -> Result<Value, String> {
	info!("[Track] Frontend command: {}", command);

	trace!("[Track] Frontend command args: {:?}", args);

	let effect_result = create_effect_for_frontend_command(&app_handle, &window, &command, args);

	match effect_result {
		Ok(effect) => {
			// log::debug!("[Track] Running effect for frontend command: {}", command);

			runtime.inner().run(effect).await.map_err(|e| {
				error!("[Track] Error running frontend effect '{}': {}", command, e);

				// Convert CommonError to String for Tauri
				e.to_string()
			})
		},

		Err(e) => {
			error!("[Track] Error creating effect for frontend command '{}': {}", command, e);

			Err(e)
		},
	}
}

// --- Sidecar Request/Notification Dispatcher ---
pub async fn dispatch_sidecar_request<R:TauriRuntime>(
	app:AppHandle<R>,

	// May be needed by some direct handlers or RPC methods
	window:Window<R>,

	// Renamed for clarity
	runtime_state:State<'_, Arc<AppRuntime>>,

	sidecar_id:String,

	// Raw JSON message { "method": "...", "params": [...] }
	request:Value,
) -> Result<Value, String> {
	let method = request.get("method").and_then(Value::as_str).unwrap_or("");

	// Keep as Value initially
	let params_value = request.get("params").cloned().unwrap_or(Value::Null);

	info!(
		"[Track] Sidecar request/notification from '{}': Method='{}'",
		sidecar_id, method
	);

	trace!(
		"[Track] Sidecar params (type='{:?}'): {}...",
		params_value.kind(),
		params_value.to_string().chars().take(100).collect::<String>()
	);

	// --- Prioritize Direct Handling for Specific Notifications ---
	if method.starts_with("terminal_") {
		debug!("[Track] Routing terminal notification directly: {}", method);

		return match method {
			"terminal_setEnvironmentVariable" => {
				handlers::terminal::handle_set_environment_variable(app, params_value).await
			},

			"terminal_deleteEnvironmentVariable" => {
				handlers::terminal::handle_delete_environment_variable(app, params_value).await
			},

			"terminal_clearEnvironmentVariableCollection" => {
				handlers::terminal::handle_clear_environment_variable_collection(app, params_value).await
			},

			_ => {
				warn!("[Track] Unknown direct terminal notification: {}", method);

				Err(format!("Unknown direct terminal notification: {}", method))
			},
		};
	}

	match method {
		"$onWillActivateExtension"
		| "$onDidActivateExtension"
		| "$onExtensionActivationError"
		| "$onExtensionRuntimeError" => {
			debug!("[Track] Routing ext status notification directly: {}", method);

			// handlers::extension_status expects params as Value::Array
			let params_array_for_handler = params_value
				.as_array()
				.map(Value::Array)
				.unwrap_or_else(|| Value::Array(vec![params_value.clone()]));

			return handlers::extension_status::handle_ext_host_status(app, method, params_array_for_handler).await;
		},

		// Continue to effect creation or RPC/direct handlers
		_ => {},
	}

	// --- Attempt Effect Creation for RPC Requests ($methods) ---
	// Most RPC methods should ideally map to effects.
	// `create_effect_for_sidecar_request` expects params as `Vec<Value>`
	let params_vec = params_value.as_array().cloned().unwrap_or_else(|| vec![params_value.clone()]);

	// Clone params_vec for fallback
	let effect_result = create_effect_for_sidecar_request(&sidecar_id, method, params_vec.clone());

	match effect_result {
		Ok(effect) => {
			// log::debug!("[Track] Running effect for sidecar request: {}", method);

			return runtime_state.inner().run(effect).await.map_err(|e| {
				error!("[Track] Error running sidecar effect '{}': {}", method, e);

				e.to_string()
			});
		},

		Err(ref e_str) if e_str.starts_with("Direct handler") || e_str.starts_with("RPC handler") => {
			// Fall through to RPC struct methods or direct handlers
			debug!("[Track] Fallback for sidecar request '{}': {}", method, e_str);
		},

		Err(e_str) => {
			error!("[Track] Error creating effect for sidecar request '{}': {}", method, e_str);

			return Err(e_str);
		},
	};

	// --- Fallback to RPC Struct Methods or Direct Handlers ---
	// This section is reached if create_effect_for_sidecar_request signals a
	// fallback. The method name includes '$' for RPC methods.
	// `params_value` is used here, as handlers/rpc methods might expect
	// Value::Array or Value::Object.
	let rpc_runtime_clone = runtime_state.inner().clone();

	match method {
		// --- RPC methods handled by rpc.rs structs ---
		"$executeCommand" | "$getCommands" | "$registerCommand" | "$unregisterCommand" => {
			let handler = rpc::MainThreadCommandsHandler { app_handle:app.clone(), runtime:rpc_runtime_clone };

			match method {
				"$executeCommand" => handler.executeCommand(params_value).await,

				"$getCommands" => handler.getCommands(params_value).await,

				"$registerCommand" => handler.registerCommand(params_value).await,

				"$unregisterCommand" => handler.unregisterCommand(params_value).await,

				_ => unreachable!(),
			}
		},

		"$resolveWorkspaceFolder" => {
			// Note: $getWorkspaceFolders is an effect
			let handler = rpc::MainThreadWorkspaceHandler { app_handle:app.clone(), runtime:rpc_runtime_clone };

			handler.resolveWorkspaceFolder(params_value).await
		},

		// $getConfiguration, $updateConfigurationOption, etc. are now effects.
		// $getValue (storage), $setValue (storage) are effects.
		// $getPassword (secrets), etc. are effects.
		"$log" => {
			// MainThreadLogHandler
			let handler = rpc::MainThreadLogHandler { app_handle:app.clone(), runtime:rpc_runtime_clone };

			handler.log(params_value).await
		},

		// Language Features: Some registrations might fallback to RPC struct methods if not effects
		"$registerHoverProvider" | "$registerCompletionsProvider" | "$registerDefinitionSupport" | "$unregister" => {
			let handler = rpc::MainThreadLanguageFeaturesHandler { app_handle:app.clone(), runtime:rpc_runtime_clone };

			match method {
				"$registerHoverProvider" => handler.registerHoverProvider(params_value).await,

				"$registerCompletionsProvider" => handler.registerCompletionsProvider(params_value).await,

				"$registerDefinitionSupport" => handler.registerDefinitionSupport(params_value).await,

				"$unregister" => handler.unregister(params_value).await,

				// TODO: Add all other $register...Provider methods here
				_ => unreachable!(),
			}
		},

		"$showMessage" => {
			let handler = rpc::MainThreadMessageHandler { app_handle:app.clone(), runtime:rpc_runtime_clone };

			handler.showMessage(params_value).await
		},

		"$showOpenDialog" | "$showSaveDialog" => {
			let handler = rpc::MainThreadDialogsHandler { app_handle:app.clone(), runtime:rpc_runtime_clone };

			match method {
				"$showOpenDialog" => handler.showOpenDialog(params_value).await,

				"$showSaveDialog" => handler.showSaveDialog(params_value).await,

				_ => unreachable!(),
			}
		},

		"$focusWindow" => {
			let handler = rpc::MainThreadWindowHandler { app_handle:app.clone(), runtime:rpc_runtime_clone };

			handler.focusWindow(params_value).await
		},

		"$setEntry" | "$disposeEntry" => {
			// Status Bar
			let handler = rpc::MainThreadStatusBarHandler { app_handle:app.clone(), runtime:rpc_runtime_clone };

			match method {
				"$setEntry" => handler.setEntry(params_value).await,

				"$disposeEntry" => handler.disposeEntry(params_value).await,

				_ => unreachable!(),
			}
		},

		// --- Direct Handlers (typically for notifications or simple RPCs not mapped to effects) ---
		// Workspace FS API (handled by MainThreadFileSystemApiHandler in rpc.rs)
		"$stat" | "$readDirectory" | "$readFile" | "$writeFile" | "$createDirectory" | "$delete" | "$rename"
		| "$copy" => {
			let handler = rpc::MainThreadFileSystemApiHandler { app_handle:app.clone(), runtime:rpc_runtime_clone };

			match method {
				"$stat" => handler.stat(params_value).await,

				"$readDirectory" => handler.read_directory(params_value).await,

				"$readFile" => handler.read_file(params_value).await,

				"$writeFile" => handler.write_file(params_value).await,

				// TODO: Add createDirectory, delete, rename, copy to MainThreadFileSystemApiHandler
				_ => {
					Err(format!(
						"Method {} part of FileSystemApi but not implemented in RPC handler struct yet.",
						method
					))
				},
			}
		},

		// Workspace Info API (findFiles, others are effects or RPC calls)
		"$findFiles" => handlers::workspace::handle_find_files(app, params_value).await,

		// Document Operations (most are effects, but some might be direct if complex)
		"$tryOpenDocument" => handlers::documents::handle_try_open_document(app, params_value).await,

		"$tryCreateDocument" => handlers::documents::handle_try_create_document(app, params_value).await,

		"$trySaveDocument" => {
			// Needs specific param parsing
			let uri_val = params_value
				.as_array()
				.and_then(|a| a.get(0))
				.cloned()
				.ok_or_else(|| format!("Missing URI for {}", method))?;

			handlers::documents::handle_try_save_document(app, uri_val).await
		},

		"$trySaveDocumentAs" => {
			let uri_val = params_value
				.as_array()
				.and_then(|a| a.get(0))
				.cloned()
				.ok_or_else(|| format!("Missing URI for {}", method))?;

			handlers::documents::handle_try_save_document_as(app, uri_val).await
		},

		"$saveAll" => {
			let include_untitled = params_value
				.as_array()
				.and_then(|a| a.get(0))
				.and_then(Value::as_bool)
				.unwrap_or(true);

			handlers::documents::handle_save_all(app, include_untitled).await
		},

		// Output API
		"$register" | "$append" | "$clear" | "$replace" | "$reveal" | "$close" | "$dispose" => {
			// Assuming params_value is Value::Array for these direct handlers
			match method {
				"$register" => handlers::output::handle_register(app, params_value).await,

				"$append" => handlers::output::handle_append(app, params_value).await,

				"$clear" => handlers::output::handle_clear(app, params_value).await,

				"$replace" => handlers::output::handle_replace(app, params_value).await,

				"$reveal" => handlers::output::handle_reveal(app, params_value).await,

				"$close" => handlers::output::handle_close(app, params_value).await,

				"$dispose" => handlers::output::handle_dispose(app, params_value).await,

				_ => unreachable!(),
			}
		},

		// Diagnostics API
		"$changeMany" | "$getDiagnostics" => {
			// "$clear" for diagnostics handled by output $clear due to ambiguity
			match method {
				"$changeMany" => handlers::diagnostics::handle_change_many(app, params_value).await,

				"$getDiagnostics" => handlers::diagnostics::handle_get_diagnostics(app, params_value).await,

				_ => unreachable!(),
			}
		},

		// Terminal RPC (if not effects)
		"$createTerminal" => {
			handlers::terminal::handle_create_terminal(
				app,
				params_value.as_array().and_then(|a| a.get(0)).cloned().unwrap_or(Value::Null),
			)
			.await
		},

		"$show" | "$hide" | "$sendText" => {
			// Terminal show, hide, sendText
			match method {
				"$show" => handlers::terminal::handle_show(app, params_value).await,

				"$hide" => handlers::terminal::handle_hide(app, params_value).await,

				"$sendText" => handlers::terminal::handle_send_text(app, params_value).await,

				_ => unreachable!(),
			}
		},

		_ => {
			error!(
				"[Track] Method '{}' from sidecar '{}' has no effect mapping and no explicit RPC/direct handler.",
				method, sidecar_id
			);

			Err(format!("Unknown or unhandled sidecar request method: {}", method))
		},
	}
}

// --- Effect Creation Helpers ---
fn create_effect_for_frontend_command<R:TauriRuntime>(
	// Mark as unused if not needed by any effect creation here
	_app_handle:&AppHandle<R>,

	// Mark as unused
	_window:&Window<R>,

	command:&str,

	args:Value,
) -> Result<ActionEffect<Arc<AppRuntime>, CommonError, Value>, String> {
	let arg_err = |name:&str| -> String { format!("Missing or invalid '{}' param for command '{}'", name, command) };

	let get_str_arg = |val:&Value, key:&str| {
		val.get(key)
			.and_then(Value::as_str)
			.map(String::from)
			.ok_or_else(|| arg_err(key))
	};

	let get_path_arg = |val:&Value, key:&str| Ok(PathBuf::from(get_str_arg(val, key)?));

	let get_i64_arg = |val:&Value, key:&str| val.get(key).and_then(Value::as_i64).ok_or_else(|| arg_err(key));

	let get_vec_arg =
		|val:&Value, key:&str| val.get(key).and_then(Value::as_array).cloned().ok_or_else(|| arg_err(key));

	let get_bool_arg = |val:&Value, key:&str| val.get(key).and_then(Value::as_bool).ok_or_else(|| arg_err(key));

	match command {
		Land_Echo::REQUEST_READ_FILE => Ok(fs_effects::read_file(get_path_arg(&args, "path")?)),

		Land_Echo::REQUEST_WRITE_FILE => {
			Ok(fs_effects::write_file_string(
				get_path_arg(&args, "path")?,
				get_str_arg(&args, "content")?,
				get_bool_arg(&args, "create")?,
				get_bool_arg(&args, "overwrite")?,
			))
		},

		Land_Echo::REQUEST_NEW_FILE => {
			Ok(fs_effects::create_file(
				get_path_arg(&args, "parentDir")?.join(get_str_arg(&args, "name")?),
			))
		},

		Land_Echo::REQUEST_NEW_FOLDER => {
			Ok(fs_effects::create_directory(
				get_path_arg(&args, "parentDir")?.join(get_str_arg(&args, "name")?),
				true,
			))
			// recursive=true for new folder UI
		},

		Land_Echo::REQUEST_DELETE_PATH => {
			Ok(fs_effects::delete(
				get_path_arg(&args, "path")?,
				args.get("recursive").and_then(Value::as_bool).unwrap_or(true),
				args.get("useTrash").and_then(Value::as_bool).unwrap_or(false),
			))
		},

		Land_Echo::REQUEST_RENAME_PATH => {
			let old_path = get_path_arg(&args, "oldPath")?;

			let new_name = get_str_arg(&args, "newName")?;

			let parent = old_path
				.parent()
				.ok_or_else(|| format!("Cannot get parent for rename: {}", old_path.display()))?;

			Ok(fs_effects::rename(
				old_path,
				parent.join(new_name),
				args.get("overwrite").and_then(Value::as_bool).unwrap_or(false),
			))
		},

		Land_Echo::REQUEST_COPY_PATH => {
			let source_path = get_path_arg(&args, "sourcePath")?;

			let target_parent_dir = get_path_arg(&args, "targetParentDir")?;

			let new_name = get_str_arg(&args, "newName")?;

			let target_path = target_parent_dir.join(new_name);

			Ok(fs_effects::copy(
				source_path,
				target_path,
				args.get("overwrite").and_then(Value::as_bool).unwrap_or(false),
			))
		},

		Land_Echo::REQUEST_SAVE_FILE => {
			let uri = Url::parse(&get_str_arg(&args, "uri")?).map_err(|e| format!("Invalid URI for save: {}", e))?;

			Ok(documents_effects::try_save(uri))
		},

		Land_Echo::REQUEST_SAVE_FILE_AS => {
			let original_uri = Url::parse(&get_str_arg(&args, "originalUri")?)
				.map_err(|e| format!("Invalid original URI for save as: {}", e))?;

			let new_target_uri_opt = args
				.get("newTargetUri")
				.and_then(Value::as_str)
				.map(|s| Url::parse(s))
				.transpose()
				.map_err(|e| format!("Invalid new target URI for save as: {}", e))?;

			Ok(documents_effects::try_save_as(original_uri, new_target_uri_opt))
		},

		Land_Echo::REQUEST_APPLY_EDITOR_CHANGES => {
			let uri = Url::parse(&get_str_arg(&args, "uri")?).map_err(|e| format!("Invalid URI: {}", e))?;

			let version_id = get_i64_arg(&args, "versionId")?;

			// DTO as Value
			let changes_val = args.get("changes").cloned().ok_or_else(|| arg_err("changes"))?;

			let is_dirty = get_bool_arg(&args, "isDirty")?;

			let is_undoing = get_bool_arg(&args, "isUndoing").unwrap_or(false);

			let is_redoing = get_bool_arg(&args, "isRedoing").unwrap_or(false);

			Ok(documents_effects::apply_changes(
				uri,
				version_id,
				changes_val,
				is_dirty,
				is_undoing,
				is_redoing,
			))
		},

		Land_Echo::REQUEST_OPEN_FILE => {
			let uri_components = args.get("uriComponents").cloned().ok_or_else(|| arg_err("uriComponents"))?;

			let language_id_opt = args.get("languageId").and_then(Value::as_str).map(String::from);

			let content_opt = args.get("content").and_then(Value::as_str).map(String::from);

			Ok(documents_effects::try_open(uri_components, language_id_opt, content_opt))
		},

		Land_Echo::REQUEST_PROXY_EXT_HOST_CALL => {
			Ok(ipc_effects::proxy_call_to_sidecar(
				"cocoon-main".to_string(),
				args.get("callData").cloned().ok_or_else(|| arg_err("callData"))?,
			))
		},

		Land_Echo::REQUEST_ESTABLISH_HOST_CONNECTION => {
			Ok(ipc_effects::establish_host_connection("cocoon-main".to_string()))
		},

		Land_Echo::REQUEST_WS_SEND | Land_Echo::REQUEST_WS_CONNECT => {
			Err("WebSocket effects not implemented".to_string())
		},

		_ => Err(format!("Unknown frontend command: {}", command)),
	}
}

fn create_effect_for_sidecar_request(
	sidecar_id:&str,

	method:&str,

	// Method params as a Vec<Value>
	params:Vec<Value>,
) -> Result<ActionEffect<Arc<AppRuntime>, CommonError, Value>, String> {
	let param_err =
		|name:&str, idx:usize| format!("Missing/invalid '{}' param at index {} for method '{}'", name, idx, method);

	let get_str_param = |p_vec:&Vec<Value>, idx:usize, name:&str| {
		p_vec
			.get(idx)
			.and_then(Value::as_str)
			.map(String::from)
			.ok_or_else(|| param_err(name, idx))
	};

	let get_u32_param = |p_vec:&Vec<Value>, idx:usize, name:&str| {
		p_vec
			.get(idx)
			.and_then(Value::as_u64)
			.map(|v| v as u32)
			.ok_or_else(|| param_err(name, idx))
	};

	// Returns Option<Value>
	let get_opt_param = |p_vec:&Vec<Value>, idx:usize| p_vec.get(idx).cloned();

	match method {



		// --- Configuration ---
		"$getConfiguration" => {



			let section = params.get(0).and_then(Value::as_str).map(String::from);

			let overrides_val = get_opt_param(params, 1).unwrap_or(Value::Null);

			let scope_to_language = params.get(2).and_then(Value::as_bool);

			Ok(config_effects::get_configuration(section, overrides_val, scope_to_language))
		},

		"$updateConfigurationOption" => {



			let target_num = params.get(0).and_then(Value::as_u64).ok_or_else(|| param_err("target", 0))? as u32;

			// ConfigurationTarget enum mapping handled by config_effects::update_configuration
			let key = get_str_param(params, 1, "key")?;

			let value = get_opt_param(params, 2).ok_or_else(|| param_err("value", 2))?;

			let overrides_val = get_opt_param(params, 3).unwrap_or(Value::Null);

			// Deserialization of overrides handled by config_effects::update_configuration
			let scope_to_language = params.get(4).and_then(Value::as_bool);

			Ok(config_effects::update_configuration(key, value, target_num, overrides_val, scope_to_language))
		},

		 // Similar to update, but value is Value::Null
		"$removeConfigurationOption" => {



			let target_num = params.get(0).and_then(Value::as_u64).ok_or_else(|| param_err("target", 0))? as u32;

			let key = get_str_param(params, 1, "key")?;

			let overrides_val = get_opt_param(params, 2).unwrap_or(Value::Null);

			let scope_to_language = params.get(3).and_then(Value::as_bool);

			Ok(config_effects::update_configuration(key, Value::Null, target_num, overrides_val, scope_to_language))
		},

		"$inspect" => Ok(config_effects::inspect_configuration(get_str_param(params,0,"key")?)),


		// --- Workspace Info ---
		"$getWorkspaceFolders" => Ok(workspace_effects::get_workspace_folders()),

        "$requestWorkspaceTrust" => {



            let options_val = get_opt_param(params, 0);

            Ok(workspace_effects::request_trust(options_val))
        }


		// --- Commands ---
		// $executeCommand, $getCommands, $registerCommand, $unregisterCommand are fallback to RPC struct
		// If they were effects:
		// "$executeCommand" => Ok(command_effects::execute_command(get_str_param(params,0,"commandId")?, Value::Array(params.get(1..).unwrap_or_default().to_vec()))),


		// --- Storage ---
		 // Params: Value::Array([{scope: number, key: string}])
		"$getValue" => {



			let target_obj = get_opt_param(params, 0).ok_or_else(|| param_err("target object", 0))?;

			Ok(storage_effects::get_storage_item(target_obj))
		}

		 // Params: Value::Array([{scope: number, key: string}, value_to_set])
		"$setValue" => {



			let target_obj = get_opt_param(params, 0).ok_or_else(|| param_err("target object", 0))?;

			let value_to_set = get_opt_param(params, 1).ok_or_else(|| param_err("value", 1))?;

			Ok(storage_effects::set_storage_item(target_obj, value_to_set))
		}


		// --- Secrets ---
		"$getPassword" => Ok(secrets_effects::get_secret(get_str_param(params,0,"extensionId")?, get_str_param(params,1,"key")?)),

		"$setPassword" => Ok(secrets_effects::store_secret(get_str_param(params,0,"extensionId")?, get_str_param(params,1,"key")?, get_str_param(params,2,"value")?)),

		"$deletePassword" => Ok(secrets_effects::delete_secret(get_str_param(params,0,"extensionId")?, get_str_param(params,1,"key")?)),


		// --- Language Features (Registration part) ---
		// $registerHoverProvider, $registerCompletionsProvider, $unregister are fallback to RPC struct
		// If they were effects directly created here:
		// "$registerHoverProvider" => {



		 // Note: Cocoon sends handle as param 0, selector as 1
		//     let selector = get_opt_param(params, 1).ok_or_else(|| param_err("selector", 1))?;

		//     Ok(language_feature_effects::register_hover_provider(selector, sidecar_id.to_string()))
		// }


		// --- UI (some can be effects, others need fallback for complex interaction) ---
		// $showMessage, $showOpenDialog, $showSaveDialog are fallback to RPC struct
		// $focusWindow, $setEntry, $disposeEntry are fallback to RPC struct

		// --- Notifications / Logging (No-op effects or direct handlers) ---
		// $log, $onWillActivateExtension, etc. are routed directly or via RPC struct.

		// --- Methods explicitly marked for direct handler or RPC struct fallback ---
		"$executeCommand" | "$getCommands" | "$registerCommand" | "$unregisterCommand" |
         // $getWorkspaceFolders is effect
		"$resolveWorkspaceFolder" |
         // Most lang features are RPC
		"$registerHoverProvider" | "$registerCompletionsProvider" | "$registerDefinitionSupport" | "$unregister" |
         // Dialogs
		"$showMessage" | "$showOpenDialog" | "$showSaveDialog" |
         // Window, Statusbar
		"$focusWindow" | "$setEntry" | "$disposeEntry" |
		 // Workspace, Docs direct
		"$findFiles" | "$tryOpenDocument" | "$tryCreateDocument" | "$trySaveDocument" | "$trySaveDocumentAs" | "$saveAll" |
		 // Output direct
		"$register" | "$append" | "$clear" | "$replace" | "$reveal" | "$close" | "$dispose" |
		 // Diagnostics direct
		"$changeMany" | "$getDiagnostics" |
		 // FS API via RPC struct
		"$stat" | "$readDirectory" | "$readFile" | "$writeFile" | "$createDirectory" | "$delete" | "$rename" | "$copy" |
         // Terminal direct
		"$createTerminal" | "$show" | "$hide" | "$sendText"
			 // Signal fallback to RPC struct method
			=> Err("RPC handler needed for this method".to_string()),


		_ => Err(format!("Unknown sidecar request method or no effect mapping: {}", method)),

	}
}
