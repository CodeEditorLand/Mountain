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
// - Prioritizing direct handling for specific notifications.
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
//   `extHost.protocol.ts` (sidecar RPC methods via Cocoon shims).
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

	// Ensure FsReader is imported for the generic effect wrapper
	fs_effects::{self, FsReader},

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

// Added rpc module
use crate::{app_state::AppState, handlers, rpc, runtime::AppRuntime, vine};

// --- Helper for Argument Parsing Errors / Structured Error String ---
fn arg_parse_error_str(method_name:&str, param_name:&str, expected_type:&str, index:Option<usize>) -> String {
	
	let mut msg = format!(
		"Track Dispatch: Missing or invalid '{}' parameter (expected {}) for method '{}'.",

		param_name, expected_type, method_name
	);
	if let Some(idx) = index {
		
		msg.push_str(&format!(" Arg index: {}.", idx));
	}

	error!("{}", msg);
	create_handler_error_string(msg, Some("EBADARG"))
}


fn create_handler_error_string(message:String, code:Option<&str>) -> String {
	
	json!({ "message": message, "code": code.unwrap_or("EUNKNOWN") }).to_string()
}


fn map_common_error_to_handler_string(e:CommonError) -> String {
	
	let (message, code) = match e {
		
		CommonError::FsNotFound(p) => (format!("File not found: {}", p.display()), "ENOENT"),

		CommonError::FsPermissionDenied(p, m) => (format!("Permission denied for '{}': {}", p.display(), m), "EACCES"),

		CommonError::FsFileExists(p) => (format!("File already exists: {}", p.display()), "EEXIST"),

		CommonError::FsNotADirectory(p) => (format!("Path is not a directory: {}", p.display()), "ENOTDIR"),

		CommonError::FsIsADirectory(p) => (format!("Path is a directory: {}", p.display()), "EISDIR"),

		CommonError::FsNotEmpty(p) => (format!("Directory not empty: {}", p.display()), "ENOTEMPTY"),

		CommonError::ConfigUpdate(_, m) => (m, "ECONFIGUPDATE"),

		CommonError::ConfigLoad(m) => (m, "ECONFIGLOAD"),

		CommonError::InvalidArg(arg_name, m) => (format!("Invalid argument '{}': {}", arg_name, m), "EBADARG"),

		CommonError::NotImplemented(feat) => (format!("Feature not implemented: {}", feat), "ENOSYS"),

		CommonError::StateLock(m) => (format!("Internal state error: {}", m), "ESTATELOCK"),

		CommonError::IpcError(m) => (format!("IPC Error: {}", m), "EIPC"),

		// More specific than just EUNKNOWN
		_ => (e.to_string(), "EUNKNOWN_EFFECT_ERROR"),

	};
	create_handler_error_string(message, Some(code))
}


// --- Frontend Command Dispatcher ---
#[command]
pub async fn dispatch_command<R:TauriRuntime>(
	app_handle:AppHandle<R>,

	window:Window<R>,

	runtime:State<'_, Arc<AppRuntime>>,

	command:String,

	args:Value,

) -> Result<Value, String> {
	
	info!(
		"[Track FrontendCmd] Dispatching: '{}', Args (brief): '{}...'",

		command,

		args.to_string().chars().take(100).collect::<String>()
	);

	match create_effect_for_frontend_command(&app_handle, &window, &command, args) {
		
		Ok(effect) => {
			
			debug!("[Track FrontendCmd] Running effect for: '{}'", command);
			runtime.run(effect).await.map_err(|e| {
				
				error!("[Track FrontendCmd] Error running effect for '{}': {}", command, e);
				map_common_error_to_handler_string(e)
			})
		},

		Err(e_str) => {
			
			// EffectCreationError for frontend is just String for now
			error!("[Track FrontendCmd] Error creating effect for '{}': {}", command, e_str);
			// Ensure error is structured
			Err(create_handler_error_string(e_str, None))
		},

	}

}


// --- Sidecar Request/Notification Dispatcher ---
pub async fn dispatch_sidecar_request<R: TauriRuntime>(
	app_handle: AppHandle<R>,

	window: Window<R>,

	runtime: State<'_, Arc<AppRuntime>>,

	sidecar_id: String,

	request: Value,

) -> Result<Value, String> {
	
	let method = request.get("method").and_then(Value::as_str).unwrap_or("");
	let params_val = request.get("params").cloned().unwrap_or(Value::Null);

	info!("[Track SidecarReq] From '{}': Method='{}', Params Type='{:?}'", sidecar_id, method, params_val.kind());
	trace!("[Track SidecarReq] Full Params: {:?}", params_val);

	// --- Prioritize Direct Handling for Specific Notifications from Cocoon ---
	if method.starts_with("terminal_") && method != "$createTerminal" {
		
		debug!("[Track SidecarReq] Routing terminal notification '{}' directly to handler.", method);
		return match method {
			
			"terminal_setEnvironmentVariable" => handlers::terminal::handle_set_environment_variable(app_handle, params_val).await,

			"terminal_deleteEnvironmentVariable" => handlers::terminal::handle_delete_environment_variable(app_handle, params_val).await,

			"terminal_clearEnvironmentVariableCollection" => handlers::terminal::handle_clear_environment_variable_collection(app_handle, params_val).await,

			_ => {
				
				warn!("[Track SidecarReq] Unknown direct terminal notification: {}", method);
				Err(create_handler_error_string(format!("Unknown direct terminal notification: {}", method), Some("ENOSYS")))
			}

		};
	}

	match method {
		
		// Logging and Extension Status notifications are handled by specific RPC struct methods
		"$log" | "$logExtensionHostActivation" | "$logExtensionHostRequest" => {
			
			let rpc_handler = rpc::MainThreadLogHandler { app_handle, runtime: runtime.inner().clone() };
			 // Assuming params_val is correct structure for $log (array)
			return rpc_handler.log(params_val).await;
		}

		"$onWillActivateExtension" | "$onDidActivateExtension" | "$onExtensionActivationError" | "$onExtensionRuntimeError" => {
			
			let rpc_handler = rpc::MainThreadExtensionServiceHandler { app_handle, runtime: runtime.inner().clone() };
			 // These expect array params
			let params_array = params_val.as_array().cloned().unwrap_or_default();
			return match method {
				
				"$onWillActivateExtension" => rpc_handler.onWillActivateExtension(Value::Array(params_array)).await,

				"$onDidActivateExtension" => rpc_handler.onDidActivateExtension(Value::Array(params_array)).await,

				"$onExtensionActivationError" => rpc_handler.onExtensionActivationError(Value::Array(params_array)).await,

				"$onExtensionRuntimeError" => rpc_handler.onExtensionRuntimeError(Value::Array(params_array)).await,

				_ => unreachable!(),

			};
		}

		_ => { /* Continue to effect/RPC handler logic */ }

	}


	// --- Attempt Effect Creation for RPC Requests ($methods) ---
	let params_array_for_effects = params_val.as_array().cloned().unwrap_or_else(|| vec![params_val.clone()]);
	match create_effect_for_sidecar_request(&sidecar_id, method, params_array_for_effects.clone()) {
		
		Ok(effect) => {
			
			debug!("[Track SidecarReq] Running effect for: '{}'", method);
			return runtime.run(effect).await.map_err(|e| {
				
				error!("[Track SidecarReq] Error running effect for '{}': {}", method, e);
				map_common_error_to_handler_string(e)
			});
		}

		Err(EffectCreationError::NoEffectMapping) => {
			
			debug!("[Track SidecarReq] No direct effect mapping for '{}'. Trying RPC direct handlers.", method);
		}

		Err(EffectCreationError::ParamParseError(e)) => {
			
			error!("[Track SidecarReq] Parameter parsing error for effect '{}': {}", method, e);
			return Err(create_handler_error_string(e, Some("EBADARG")));
		}

	}


	// --- Direct RPC Handler Fallback/Implementation ---
	debug!("[Track SidecarReq] Attempting direct RPC handler for: '{}'", method);
	let rpc_runtime_clone = runtime.inner().clone();
	match method {
		
		"$executeCommand" | "$getCommands" | "$registerCommand" | "$unregisterCommand" => {
			
			let handler = rpc::MainThreadCommandsHandler { app_handle, runtime: rpc_runtime_clone };
			match method {
				
				"$executeCommand" => handler.executeCommand(params_val).await,

				"$getCommands" => handler.getCommands(params_val).await,

				"$registerCommand" => handler.registerCommand(params_val).await,

				"$unregisterCommand" => handler.unregisterCommand(params_val).await,

				_ => unreachable!(),

			}

		}

		 // $getWorkspaceFolders is an effect
		"$resolveWorkspaceFolder" | "$findFiles" => {
			
			let handler = rpc::MainThreadWorkspaceHandler { app_handle, runtime: rpc_runtime_clone };
			match method {
				
				"$resolveWorkspaceFolder" => handler.resolveWorkspaceFolder(params_val).await,

				"$findFiles" => handler.findFiles(params_val).await,

				_ => unreachable!(),

			}

		}

		// Most config, storage, secrets are effects. If any fallback, add here.

		// Language Features: Some registrations fallback to RPC struct methods.
		"$registerHoverProvider" | "$registerCompletionsProvider" | "$registerDefinitionSupport" | "$unregister"
		// TODO: Add ALL $register... methods from MainThreadLanguageFeaturesShape here for fallback
		// (if they are not effects)
		=> {
			
			let handler = rpc::MainThreadLanguageFeaturesHandler { app_handle, runtime: rpc_runtime_clone };
			match method {
				
				"$registerHoverProvider" => handler.registerHoverProvider(params_val).await,

				"$registerCompletionsProvider" => handler.registerCompletionsProvider(params_val).await,

				"$registerDefinitionSupport" => handler.registerDefinitionSupport(params_val).await,

				"$unregister" => handler.unregister(params_val).await,

				_ => Err(create_handler_error_string(format!("Language feature method '{}' not fully mapped in Track fallback.", method), Some("ENOSYS"))),

			}

		}

		// UI Methods via RPC handlers
		"$showMessage" => {
			
			let handler = rpc::MainThreadMessageHandler { app_handle, runtime: rpc_runtime_clone };
			handler.showMessage(params_val).await
		}

		"$showOpenDialog" | "$showSaveDialog" => {
			
			let handler = rpc::MainThreadDialogsHandler { app_handle, runtime: rpc_runtime_clone };
			match method {
				
				"$showOpenDialog" => handler.showOpenDialog(params_val).await,

				"$showSaveDialog" => handler.showSaveDialog(params_val).await,

				_ => unreachable!(),

			}

		}

		"$focusWindow" => {
			
			let handler = rpc::MainThreadWindowHandler { app_handle, runtime: rpc_runtime_clone };
			handler.focusWindow(params_val).await
		}

		 // Status Bar
		"$setEntry" | "$disposeEntry" => {
			
			let handler = rpc::MainThreadStatusBarHandler { app_handle, runtime: rpc_runtime_clone };
			match method {
				
				"$setEntry" => handler.setEntry(params_val).await,

				"$disposeEntry" => handler.disposeEntry(params_val).await,

				_ => unreachable!(),

			}

		}

		// Workspace FS API (via rpc.rs handler)
		"$stat" | "$readDirectory" | "$readFile" | "$writeFile" | "$createDirectory" | "$delete" | "$rename" | "$copy" => {
			
			let fs_handler = rpc::MainThreadFileSystemApiHandler { app_handle, runtime: rpc_runtime_clone };
			match method {
				
				"$stat" => fs_handler.stat(params_val).await,

				"$readDirectory" => fs_handler.read_directory(params_val).await,

				"$readFile" => fs_handler.read_file(params_val).await,

				"$writeFile" => fs_handler.write_file(params_val).await,

				// TODO: Map createDirectory, delete, rename, copy in rpc.rs and call here
				_ => Err(create_handler_error_string(format!("FS API method '{}' not fully mapped in Track.", method), Some("ENOSYS"))),

			}

		}

		// Document Operations (via handlers::documents which uses effects internally)
		"$tryOpenDocument" => handlers::documents::handle_try_open_document(app_handle, params_val).await,

		"$tryCreateDocument" => handlers::documents::handle_try_create_document(app_handle, params_val).await,

		"$trySaveDocument" => {
			
			let uri_val = params_val.as_array().and_then(|a|a.get(0)).cloned().ok_or_else(|| arg_parse_error_str(method, "uriComponents", "Value", Some(0)))?;
			handlers::documents::handle_try_save_document(app_handle, uri_val).await
		}

        "$trySaveDocumentAs" => {
			
            let uri_val = params_val.as_array().and_then(|a|a.get(0)).cloned().ok_or_else(|| arg_parse_error_str(method, "uriComponents", "Value", Some(0)))?;
            handlers::documents::handle_try_save_document_as(app_handle, uri_val).await
        }

		"$saveAll" => {
			
			let include_untitled = params_val.as_array().and_then(|a|a.get(0)).and_then(Value::as_bool).unwrap_or(true);
			handlers::documents::handle_save_all(app_handle, include_untitled).await
		},

		// Output API (direct to handlers::output)
		"$register" | "$append" | "$clear" | "$replace" | "$reveal" | "$close" | "$dispose" if is_output_method(method, ¶ms_val) => {
			
			match method {
				
				"$register" => handlers::output::handle_register(app_handle, params_val).await,

				"$append" => handlers::output::handle_append(app_handle, params_val).await,

				"$clear" => handlers::output::handle_clear(app_handle, params_val).await,

				"$replace" => handlers::output::handle_replace(app_handle, params_val).await,

				"$reveal" => handlers::output::handle_reveal(app_handle, params_val).await,

				"$close" => handlers::output::handle_close(app_handle, params_val).await,

				"$dispose" => handlers::output::handle_dispose(app_handle, params_val).await,

				_ => unreachable!(),

			}

		}

		// Diagnostics API (direct to handlers::diagnostics)
		 // $clear for diagnostics is an effect
		"$changeMany" | "$getDiagnostics" => {
			
			match method {
				
				"$changeMany" => handlers::diagnostics::handle_change_many(app_handle, params_val).await,

				"$getDiagnostics" => handlers::diagnostics::handle_get_diagnostics(app_handle, params_val).await,

				_ => unreachable!(),

			}

		}

		// Terminal RPC API (direct to handlers::terminal or rpc::MainThreadTerminalServiceHandler)
		"$createTerminal" | "$show" | "$hide" | "$sendText" if is_terminal_method_rpc(method) => {
			
			// Using handlers::rpc struct for these as they are more RPC-like
			let rpc_handler = rpc::MainThreadTerminalServiceHandler { app_handle, runtime: rpc_runtime_clone };
			match method {
				
				"$createTerminal" => rpc_handler.createTerminal(params_val).await,

				"$show" => rpc_handler.show(params_val).await,

				"$hide" => rpc_handler.hide(params_val).await,

				"$sendText" => rpc_handler.sendText(params_val).await,

				_ => unreachable!(),

			}

		}

		"$dispose" if params_val.as_array().map_or(false, |arr| arr.get(0).and_then(Value::as_u64).is_some()) => {
			
			let rpc_handler = rpc::MainThreadTerminalServiceHandler { app_handle, runtime: rpc_runtime_clone };
			info!("[Track SidecarReq] Assuming '$dispose' is for Terminal due to u64 param.");
			rpc_handler.dispose(params_val).await
		}

		_ => {
			
			error!("[Track SidecarReq] Method '{}' has no effect mapping AND no explicit RPC/direct handler.", method);
			Err(create_handler_error_string(format!("Method '{}' not implemented or mapped in Mountain's Track dispatcher.", method), Some("ENOSYS")))
		}

	}

}


// Helper functions to disambiguate overloaded method names like $dispose
// Now takes params_val to help disambiguate based on expected param structure
// if needed.
fn is_output_method(method_name:&str, _params_val:&Value) -> bool {
	
	matches!(
		method_name,

		"$register" | "$append" | "$clear" | "$replace" | "$reveal" | "$close" | "$dispose"
	)
	// For $dispose, if params[0] is string (channelId), it's likely output.
	// If params[0] is u64 (terminalId), it's terminal.
}


fn is_terminal_method_rpc(method_name:&str) -> bool {
	
	matches!(method_name, "$createTerminal" | "$show" | "$hide" | "$sendText")
	// Note: $dispose for terminal usually takes a u64 handle.
}


// Enum to differentiate errors from effect creation
enum EffectCreationError {
	
	NoEffectMapping,

	ParamParseError(String),

}



/// Creates an ActionEffect for commands originating from the frontend.
fn create_effect_for_frontend_command<R:TauriRuntime>(
	 // Mark as unused if no effect creation needs it yet
	_app_handle:&AppHandle<R>,

	_window:&Window<R>,

	command:&str,

	args:Value,

) -> Result<ActionEffect<Arc<AppRuntime>, CommonError, Value>, String> {
	
	// Return String for error
	let param_err_fn = |name:&str| -> String { arg_parse_error_str(command, name, "specific type", None) };
	let get_str_arg = |key:&str| {
		
		args.get(key)
			.and_then(Value::as_str)
			.map(String::from)
			.ok_or_else(|| param_err_fn(key))
	};
	let get_path_arg = |key:&str| get_str_arg(key).map(PathBuf::from);
	let get_i64_arg = |key:&str| args.get(key).and_then(Value::as_i64).ok_or_else(|| param_err_fn(key));
	let get_vec_arg = |key:&str| {
		
		args.get(key)
			.and_then(Value::as_array)
			.cloned()
			.ok_or_else(|| param_err_fn(key))
	};
	let get_bool_arg = |key:&str, default:bool| args.get(key).and_then(Value::as_bool).unwrap_or(default);

	trace!("[Track CreateEffect Frontend] Command='{}', Args='{:?}'", command, args);

	match command {
		
		Land_Echo::REQUEST_READ_FILE => {
			
			let path = get_path_arg("path")?;
			let effect = fs_effects::read_file(path);
			Ok(ActionEffect::new(Arc::new(move |env_accessor| {
				
				let effect_clone = effect.clone();
				Box::pin(async move {
					
					 // Generic access
					let fs_reader_env:Arc<dyn FsReader + Send + Sync> = env_accessor.require();
					fs_reader_env
						.run_effect(effect_clone)
						.await
						.map(|bytes| json!(base64::encode(bytes)))
				})
			})))
		},

		Land_Echo::REQUEST_WRITE_FILE => {
			
			Ok(fs_effects::write_file_string(
				get_path_arg("path")?,

				get_str_arg("content")?,

				 // Default create to true from frontend
				get_bool_arg("create", true),   
				 // Default overwrite to true
				get_bool_arg("overwrite", true),

			))
		},

		Land_Echo::REQUEST_NEW_FILE => {
			
			Ok(fs_effects::create_file(get_path_arg("parentDir")?.join(get_str_arg("name")?)))
		},

		Land_Echo::REQUEST_NEW_FOLDER => {
			
			Ok(fs_effects::create_directory(
				get_path_arg("parentDir")?.join(get_str_arg("name")?),

				true,

			))
		},

		Land_Echo::REQUEST_DELETE_PATH => {
			
			Ok(fs_effects::delete(
				get_path_arg("path")?,

				get_bool_arg("recursive", true),

				get_bool_arg("useTrash", false),

			))
		},

		Land_Echo::REQUEST_RENAME_PATH => {
			
			let old_path = get_path_arg("oldPath")?;
			let new_name = get_str_arg("newName")?;
			let parent = old_path
				.parent()
				.ok_or_else(|| format!("Cannot get parent for rename: {}", old_path.display()))?;
			Ok(fs_effects::rename(
				old_path,

				parent.join(new_name),

				get_bool_arg("overwrite", false),

			))
		},

		Land_Echo::REQUEST_COPY_PATH => {
			
			let source_path = get_path_arg("sourcePath")?;
			let target_parent_dir = get_path_arg("targetParentDir")?;
			let new_name = get_str_arg("newName")?;
			Ok(fs_effects::copy(
				source_path,

				target_parent_dir.join(new_name),

				get_bool_arg("overwrite", false),

			))
		},

		Land_Echo::REQUEST_SAVE_FILE => {
			
			let uri = Url::parse(&get_str_arg("uri")?).map_err(|e| format!("Invalid URI for save: {}", e))?;
			 // Returns bool
			Ok(documents_effects::try_save(uri))
		},

		Land_Echo::REQUEST_SAVE_FILE_AS => {
			
			let original_uri = Url::parse(&get_str_arg("originalUri")?)
				.map_err(|e| format!("Invalid original URI for save as: {}", e))?;
			let new_target_uri_opt = args
				.get("newTargetUri")
				.and_then(Value::as_str)
				.map(|s| Url::parse(s))
				.transpose()
				.map_err(|e| format!("Invalid new target URI for save as: {}", e))?;
			 // Returns Option<Url>
			Ok(documents_effects::try_save_as(original_uri, new_target_uri_opt))
		},

		Land_Echo::REQUEST_APPLY_EDITOR_CHANGES => {
			
			let uri = Url::parse(&get_str_arg("uri")?).map_err(|e| format!("Invalid URI: {}", e))?;
			let version_id = get_i64_arg("versionId")?;
			let changes_val = args.get("changes").cloned().ok_or_else(|| param_err_fn("changes"))?;
			let is_dirty = get_bool_arg("isDirty", true);
			let is_undoing = get_bool_arg("isUndoing", false);
			let is_redoing = get_bool_arg("isRedoing", false);
			Ok(documents_effects::apply_changes(
				uri,

				version_id,

				changes_val,

				is_dirty,

				is_undoing,

				is_redoing,

			 // Returns ()
			))
		},

		Land_Echo::REQUEST_OPEN_FILE => {
			
			let uri_components = args
				.get("uriComponents")
				.cloned()
				.ok_or_else(|| param_err_fn("uriComponents"))?;
			let language_id_opt = args.get("languageId").and_then(Value::as_str).map(String::from);
			let content_opt = args.get("content").and_then(Value::as_str).map(String::from);
			 // Returns Url
			Ok(documents_effects::try_open(uri_components, language_id_opt, content_opt))
		},

		Land_Echo::REQUEST_PROXY_EXT_HOST_CALL => {
			
			Ok(ipc_effects::proxy_call_to_sidecar(
				"cocoon-main".to_string(),

				args.get("callData").cloned().ok_or_else(|| param_err_fn("callData"))?,

			))
		},

		Land_Echo::REQUEST_ESTABLISH_HOST_CONNECTION => {
			
			Ok(ipc_effects::establish_host_connection("cocoon-main".to_string()))
		},

		Land_Echo::REQUEST_WS_SEND | Land_Echo::REQUEST_WS_CONNECT => {
			
			warn!(
				"[Track CreateEffect Frontend] WebSocket command '{}' received, but effects not implemented.",

				command
			);
			Err(format!("WebSocket command '{}' not implemented via effects yet.", command))
		},

		_ => {
			
			error!("[Track CreateEffect Frontend] Unknown command: {}", command);
			Err(format!("Unknown frontend command for effect creation: {}", command))
		},

	}

}



/// Attempts to create an ActionEffect for RPC requests originating from a
/// sidecar.
fn create_effect_for_sidecar_request(
	sidecar_id:&str,

	method:&str,

	params:Vec<Value>,

) -> Result<ActionEffect<Arc<AppRuntime>, CommonError, Value>, EffectCreationError> {
	
	let param_err_fn = |name:&str, idx:usize| {
		
		EffectCreationError::ParamParseError(arg_parse_error_str(method, name, "specific type", Some(idx)))
	};
	let get_str_param = |idx:usize, name:&str| {
		
		params
			.get(idx)
			.and_then(Value::as_str)
			.map(String::from)
			.ok_or_else(|| param_err_fn(name, idx))
	};
	let get_u32_param = |idx:usize, name:&str| {
		
		params
			.get(idx)
			.and_then(Value::as_u64)
			.map(|v| v as u32)
			.ok_or_else(|| param_err_fn(name, idx))
	};
	let get_opt_param = |idx:usize| params.get(idx).cloned();

	trace!(
		"[Track CreateEffect Sidecar] Method='{}', NumParams={}, Sidecar='{}'",

		method,

		params.len(),

		sidecar_id
	);

	match method {
		
		// --- Configuration ---
		"$getConfiguration" => {
			
			let section = params.get(0).and_then(Value::as_str).map(String::from);
			let overrides_val = get_opt_param(1).unwrap_or(Value::Null);
			let scope_to_language = params.get(2).and_then(Value::as_bool);
			Ok(config_effects::get_configuration(section, overrides_val, scope_to_language))
		}

		"$updateConfigurationOption" => {
			
			let target_num = get_opt_param(0).and_then(|v|v.as_u64()).ok_or_else(|| param_err_fn("target",0))? as u32;
			let key = get_str_param(1, "key")?;
			let value = get_opt_param(2).ok_or_else(|| param_err_fn("value", 2))?;
			let overrides_val = get_opt_param(3).unwrap_or(Value::Null);
			Ok(config_effects::update_configuration(key, value, target_num, overrides_val, params.get(4).and_then(Value::as_bool)))
		}

		"$removeConfigurationOption" => {
			
			let target_num = get_opt_param(0).and_then(|v|v.as_u64()).ok_or_else(|| param_err_fn("target",0))? as u32;
			let key = get_str_param(1, "key")?;
			let overrides_val = get_opt_param(2).unwrap_or(Value::Null);
			Ok(config_effects::update_configuration(key, Value::Null, target_num, overrides_val, params.get(3).and_then(Value::as_bool)))
		}

		"$inspect" => Ok(config_effects::inspect_configuration(get_str_param(0, "key")?)),


		// --- Workspace Info ---
		"$getWorkspaceFolders" => Ok(workspace_effects::get_workspace_folders()),

        "$requestWorkspaceTrust" => Ok(workspace_effects::request_workspace_trust(get_opt_param(0))),

 // $resolveWorkspaceFolder, $findFiles are direct RPC
		

		// --- Commands ---
		// These are handled by RPC struct for now due to complex handler signature in commands.rs
		// If made into effects, their constructors would be here.
		// "$executeCommand" => Ok(command_effects::execute_command(get_str_param(0,"commandId")?, Value::Array(params.get(1..).unwrap_or_default().to_vec()))),

		// "$getCommands" => Ok(command_effects::get_commands()),

		// "$registerCommand" => Ok(command_effects::register_command(sidecar_id.to_string(), get_str_param(0,"commandId")?)),

		// "$unregisterCommand" => Ok(command_effects::unregister_command(sidecar_id.to_string(), get_str_param(0,"commandId")?)),


		// --- Storage ---
		"$getValue" => {
			
			let target_obj = get_opt_param(0).ok_or_else(|| param_err_fn("target object", 0))?;
			Ok(storage_effects::get_storage_item(target_obj))
		}

		"$setValue" => {
			
			let target_obj = get_opt_param(0).ok_or_else(|| param_err_fn("target object", 0))?;
			let value_to_set = get_opt_param(1).ok_or_else(|| param_err_fn("value", 1))?;
			Ok(storage_effects::set_storage_item(target_obj, value_to_set))
		}


		// --- Secrets ---
		"$getPassword" => Ok(secrets_effects::get_secret(get_str_param(0,"extensionId")?, get_str_param(1,"key")?)),

		"$setPassword" => Ok(secrets_effects::store_secret(get_str_param(0,"extensionId")?, get_str_param(1,"key")?, get_str_param(2,"value")?)),

		"$deletePassword" => Ok(secrets_effects::delete_secret(get_str_param(0,"extensionId")?, get_str_param(1,"key")?)),


		// --- Language Features Registration (Partial for MVP) ---
		// For methods routed to rpc.rs, this function should return Err(EffectCreationError::NoEffectMapping)
		// Example for $unregister, which IS an effect:
		"$unregister" => {
			
			let handle = get_u32_param(0, "handle (mountain)")?;
			Ok(language_feature_effects::unregister_provider(handle))
		}

		// Note: $register...Provider methods are routed to rpc.rs struct calls in dispatch_sidecar_request

 // --- Diagnostics ---
		
         // Heuristic for diagnostics $clear(owner: string)
		"$clear" if method == "$clear" && params.len() == 1 && params[0].is_string() => {
			
            info!("[Track CreateEffect Sidecar] Assuming '$clear' is for Diagnostics due to single string param.");
            Ok(diagnostics_effects::clear_owner_diagnostics(get_str_param(0, "owner")?))
        }



		// --- Methods explicitly routed to RPC handlers or direct handlers in dispatch_sidecar_request ---
		"$executeCommand" | "$getCommands" | "$registerCommand" | "$unregisterCommand" |
        "$resolveWorkspaceFolder" | "$findFiles" |
         // Add other $register...
		"$registerHoverProvider" | "$registerCompletionsProvider" | "$registerDefinitionSupport" |
        "$showMessage" | "$showOpenDialog" | "$showSaveDialog" |
        "$focusWindow" | "$setEntry" | "$disposeEntry" |
		"$tryOpenDocument" | "$tryCreateDocument" | "$trySaveDocument" | "$trySaveDocumentAs" | "$saveAll" |
		 // Output methods
		"$register" | "$append" | "$replace" | "$reveal" | "$close" |
		 // Diagnostics methods
		"$changeMany" | "$getDiagnostics" |
		 // FS API
		"$stat" | "$readDirectory" | "$readFile" | "$writeFile" | "$createDirectory" | "$delete" | "$rename" | "$copy" |
         // Terminal RPC
		"$createTerminal" | "$show" | "$hide" | "$sendText" |
         // Ambiguous $dispose, usually handled by context in dispatch_sidecar_request
		"$dispose"
			=> Err(EffectCreationError::NoEffectMapping),


		_ => {
			
			warn!("[Track CreateEffect Sidecar] Unknown method or no effect mapping: '{}'", method);
			Err(EffectCreationError::NoEffectMapping)
		}

	}

}


