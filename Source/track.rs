// ---------------------------------------------------------------------------------------------
// Mountain Track - Command Dispatcher (track.rs)
// --------------------------------------------------------------------------------------------
// Acts as the central routing hub for all actions within Mountain. It receives
// commands invoked from the frontend (Sky) via Tauri's `invoke` and
// requests/notifications proxied from sidecars (Cocoon) via the Vine IPC layer.
// Its primary role is to translate these incoming commands/requests into
// abstract `ActionEffect`s (defined in `Land_Common`) or route them to direct
// handler functions or RPC struct methods. Effects are then dispatched to the
// `AppRuntime` for execution.
//
// Responsibilities:
// - Implementing the Tauri `#[command]` function (`dispatch_command`).
// - Providing `dispatch_sidecar_request` called by `Vine`.
// - Parsing command/method names and arguments.
// - Prioritizing direct handling for specific notifications.
// - Mapping command/method names to `ActionEffect` constructors.
// - Falling back to `rpc.rs` methods or `handlers/*` functions.
// - Invoking `AppRuntime::run` for effects.
// - Formatting errors for the caller using shared error utilities.
//
// Key Interactions:
// - Called by Tauri (`dispatch_command`) and `Vine`
//   (`dispatch_sidecar_request`).
// - Uses `Land_Echo` (frontend) and `extHost.protocol.ts` (sidecar)
//   definitions.
// - Creates `ActionEffect`s from `Land_Common`.
// - Uses `AppRuntime` to execute effects.
// - Calls functions in `handlers::*` or methods on `rpc::*` structs.
// --------------------------------------------------------------------------------------------
use std::{path::PathBuf, sync::Arc};

use Land_Common::{
	command_effects,

	config_effects::{self, ConfigurationTarget, IConfigurationOverrides},

	diagnostics_effects,

	documents_effects,

	effect::ActionEffect,

	errors::CommonError,

	fs_effects::{self, FsReader}, // fs_effects for general FS, FsReader for generic effect wrapper
	ipc_effects,

	language_feature_effects::{self, ProviderType as CommonProviderType},

	output_effects,

	secrets_effects,

	storage_effects,

	ui_effects,

	workspace_effects,
};
use Land_Echo; // Frontend command constants
use log::{debug, error, info, trace, warn};
use serde::Deserialize;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime as TauriRuntime, State, Window, command};
use url::Url;

use crate::{
	app_state::AppState, // Though not directly used, it's context for handlers/effects
	handlers,

	handlers::error_utils, // Centralized error utilities
	rpc,

	runtime::AppRuntime,

	vine,
};

// --- Error Helper Abstraction ---
// `arg_parse_error_str` now directly uses `error_utils::rpc_param_error_string`
fn arg_parse_error_str(method_name:&str, param_name:&str, expected_type:&str, index:Option<usize>) -> String {
	error_utils::rpc_param_error_string(method_name, param_name, expected_type, index)
}

// `map_common_error_to_handler_string` uses
// `error_utils::map_common_error_to_rpc_string`
fn map_common_error_to_handler_string(e:CommonError, operation_context:&str) -> String {
	error_utils::map_common_error_to_rpc_string(e, operation_context)
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
	info!("[Track FrontendCmd] Dispatching: '{}'", command);
	trace!("[Track FrontendCmd] Args: {:?}", args);

	match create_effect_for_frontend_command(&app_handle, &window, &command, args) {
		Ok(effect) => {
			runtime.run(effect).await.map_err(|e| {
				error!("[Track FrontendCmd] Error running effect for '{}': {}", command, e);
				map_common_error_to_handler_string(e, &format!("frontend_cmd_{}", command))
			})
		},

		Err(e_str) => {
			// EffectCreationError for frontend is String (already JSON error string)
			error!("[Track FrontendCmd] Error creating effect for '{}': {}", command, e_str);
			// If e_str is already a JSON error string from rpc_param_error_string, it's
			// fine. Otherwise, wrap it. Assuming create_effect_for_frontend_command now
			// returns JSON error strings.
			if e_str.starts_with('{') && e_str.ends_with('}') {
				Err(e_str)
			} else {
				Err(error_utils::rpc_error_string(e_str, Some("EEFFECTCREATE")))
			}
		},
	}
}

// --- Sidecar Request/Notification Dispatcher ---
pub async fn dispatch_sidecar_request<R:TauriRuntime>(
	app_handle:AppHandle<R>,

	window:Window<R>,

	runtime:State<'_, Arc<AppRuntime>>,

	sidecar_id:String,

	request:Value,
) -> Result<Value, String> {
	let method = request.get("method").and_then(Value::as_str).unwrap_or("");
	let params_val = request.get("params").cloned().unwrap_or(Value::Null);

	info!("[Track SidecarReq] From '{}': Method='{}'", sidecar_id, method);
	trace!(
		"[Track SidecarReq] Params (type='{:?}'): {}...",
		params_val.kind(),
		params_val.to_string().chars().take(100).collect::<String>()
	);

	// --- Prioritize Direct Handling for Specific Notifications ---
	if method.starts_with("terminal_") && method != "$createTerminal" {
		debug!(
			"[Track SidecarReq] Routing terminal notification '{}' directly to handler.",
			method
		);
		return match method {
			"terminal_setEnvironmentVariable" => {
				handlers::terminal::handle_set_environment_variable(app_handle, params_val).await
			},

			"terminal_deleteEnvironmentVariable" => {
				handlers::terminal::handle_delete_environment_variable(app_handle, params_val).await
			},

			"terminal_clearEnvironmentVariableCollection" => {
				handlers::terminal::handle_clear_environment_variable_collection(app_handle, params_val).await
			},

			_ => {
				warn!("[Track SidecarReq] Unknown direct terminal notification: {}", method);
				Err(error_utils::rpc_error_string(
					format!("Unknown direct terminal notification: {}", method),
					Some("ENOSYS_TERM_NOTIF"),
				))
			},
		};
	}

	match method {
		"$log" | "$logExtensionHostActivation" | "$logExtensionHostRequest" => {
			let rpc_handler = rpc::MainThreadLogHandler { app_handle, runtime:runtime.inner().clone() };
			return rpc_handler.log(params_val).await;
		},

		"$onWillActivateExtension"
		| "$onDidActivateExtension"
		| "$onExtensionActivationError"
		| "$onExtensionRuntimeError" => {
			let params_array = params_val.as_array().cloned().unwrap_or_default();
			return handlers::extension_status::handle_ext_host_status(app_handle, method, Value::Array(params_array))
				.await;
		},

		_ => { /* Continue to effect/RPC handler logic */ },
	}

	// --- Attempt Effect Creation for RPC Requests ($methods) ---
	let params_array_for_effects = params_val.as_array().cloned().unwrap_or_else(|| vec![params_val.clone()]);
	match create_effect_for_sidecar_request(&sidecar_id, method, params_array_for_effects.clone()) {
		Ok(effect) => {
			debug!("[Track SidecarReq] Running effect for: '{}'", method);
			return runtime.run(effect).await.map_err(|e| {
				error!("[Track SidecarReq] Error running effect for '{}': {}", method, e);
				map_common_error_to_handler_string(e, &format!("sidecar_effect_{}", method))
			});
		},

		Err(EffectCreationError::NoEffectMapping) => {
			debug!(
				"[Track SidecarReq] No direct effect mapping for '{}'. Trying RPC/direct handlers.",
				method
			);
		},

		Err(EffectCreationError::ParamParseError(e_str)) => {
			// e_str is already a JSON error string
			error!("[Track SidecarReq] Parameter parsing error for effect '{}': {}", method, e_str);
			return Err(e_str);
		},
	}

	// --- Direct RPC Handler Fallback/Implementation ---
	debug!("[Track SidecarReq] Attempting direct RPC handler for: '{}'", method);
	let rpc_runtime_clone = runtime.inner().clone();
	match method {
		"$executeCommand" | "$getCommands" | "$registerCommand" | "$unregisterCommand" => {
			let handler = rpc::MainThreadCommandsHandler { app_handle, runtime:rpc_runtime_clone };
			match method {
				"$executeCommand" => handler.executeCommand(params_val).await,

				"$getCommands" => handler.getCommands(params_val).await,

				"$registerCommand" => handler.registerCommand(params_val).await,

				"$unregisterCommand" => handler.unregisterCommand(params_val).await,

				_ => unreachable!(),
			}
		},

		"$resolveWorkspaceFolder" => {
			let handler = rpc::MainThreadWorkspaceHandler { app_handle, runtime:rpc_runtime_clone };
			handler.resolveWorkspaceFolder(params_val).await
		},

		"$findFiles" => handlers::workspace::handle_find_files(app_handle, params_val).await,

		_ if method.starts_with("$register") && method.contains("Provider") => {
			warn!(
				"[Track SidecarReq] Language feature registration '{}' fell back to RPC handler (should be an effect).",
				method
			);
			// This path should ideally not be hit if create_effect_for_sidecar_request
			// covers all registrations. If it is, it implies a missing effect mapping.
			Err(error_utils::rpc_error_string(
				format!(
					"Fallback for lang feature registration '{}' is unexpected; should be an effect.",
					method
				),
				Some("ENOSYS_LANG_RPC"),
			))
		},

		"$showMessage" => {
			let handler = rpc::MainThreadMessageHandler { app_handle, runtime:rpc_runtime_clone };
			handler.showMessage(params_val).await
		},

		"$showOpenDialog" | "$showSaveDialog" => {
			let handler = rpc::MainThreadDialogsHandler { app_handle, runtime:rpc_runtime_clone };
			match method {
				"$showOpenDialog" => handler.showOpenDialog(params_val).await,

				"$showSaveDialog" => handler.showSaveDialog(params_val).await,

				_ => unreachable!(),
			}
		},

		"$focusWindow" => {
			let handler = rpc::MainThreadWindowHandler { app_handle, runtime:rpc_runtime_clone };
			handler.focusWindow(params_val).await
		},

		"$setEntry" | "$disposeEntry" if method == "$setEntry" || method == "$disposeEntry" => {
			let handler = rpc::MainThreadStatusBarHandler { app_handle, runtime:rpc_runtime_clone };
			match method {
				"$setEntry" => handler.setEntry(params_val).await,

				"$disposeEntry" => handler.disposeEntry(params_val).await,

				_ => unreachable!(),
			}
		},

		"$stat" | "$readDirectory" | "$readFile" | "$writeFile" | "$createDirectory" | "$delete" | "$rename"
		| "$copy" => {
			let fs_handler = rpc::MainThreadFileSystemApiHandler { app_handle, runtime:rpc_runtime_clone };
			match method {
				"$stat" => fs_handler.stat(params_val).await,

				"$readDirectory" => fs_handler.read_directory(params_val).await,

				"$readFile" => fs_handler.read_file(params_val).await,

				"$writeFile" => fs_handler.write_file(params_val).await,

				"$createDirectory" => fs_handler.create_directory(params_val).await,

				"$delete" => fs_handler.delete(params_val).await,

				"$rename" => fs_handler.rename(params_val).await,

				"$copy" => fs_handler.copy(params_val).await,

				_ => unreachable!(), // All covered by outer match
			}
		},

		"$tryOpenDocument" => handlers::documents::handle_try_open_document(app_handle, params_val).await,

		"$tryCreateDocument" => handlers::documents::handle_try_create_document(app_handle, params_val).await,

		"$trySaveDocument" => {
			let uri_val = params_val
				.as_array()
				.and_then(|a| a.get(0))
				.cloned()
				.ok_or_else(|| arg_parse_error_str(method, "uriComponents", "Value", Some(0)))?;
			handlers::documents::handle_try_save_document(app_handle, uri_val).await
		},

		"$trySaveDocumentAs" => {
			let uri_val = params_val
				.as_array()
				.and_then(|a| a.get(0))
				.cloned()
				.ok_or_else(|| arg_parse_error_str(method, "uriComponents", "Value", Some(0)))?;
			handlers::documents::handle_try_save_document_as(app_handle, uri_val).await
		},

		"$saveAll" => {
			let include_untitled = params_val
				.as_array()
				.and_then(|a| a.get(0))
				.and_then(Value::as_bool)
				.unwrap_or(true);
			handlers::documents::handle_save_all(app_handle, include_untitled).await
		},

		"$register" | "$append" | "$replace" | "$reveal" | "$close" if is_output_method_fallback(method) => {
			// Use a simpler checker for this fallback group
			match method {
				"$register" => handlers::output::handle_register(app_handle, params_val).await,

				"$append" => handlers::output::handle_append(app_handle, params_val).await,

				"$replace" => handlers::output::handle_replace(app_handle, params_val).await,

				"$reveal" => handlers::output::handle_reveal(app_handle, params_val).await,

				"$close" => handlers::output::handle_close(app_handle, params_val).await,

				_ => {
					Err(error_utils::rpc_error_string(
						format!("Output method '{}' not handled in fallback.", method),
						Some("ENOSYS_OUT_FALLBACK"),
					))
				},
			}
		},

		"$clear"
			if method == "$clear"
				&& params_val
					.as_array()
					.map_or(false, |arr| arr.get(0).map_or(false, |p| p.is_string())) =>
		{
			info!("[Track SidecarReq] Assuming '$clear' is for Output channel due to string param (fallback).");
			handlers::output::handle_clear(app_handle, params_val).await
		},

		"$dispose"
			if method == "$dispose"
				&& params_val
					.as_array()
					.map_or(false, |arr| arr.get(0).map_or(false, |p| p.is_string())) =>
		{
			info!("[Track SidecarReq] Assuming '$dispose' is for Output channel due to string param (fallback).");
			handlers::output::handle_dispose(app_handle, params_val).await
		},

		"$changeMany" | "$getDiagnostics" => {
			// $clear for diagnostics is an effect
			match method {
				"$changeMany" => handlers::diagnostics::handle_change_many(app_handle, params_val).await,

				"$getDiagnostics" => handlers::diagnostics::handle_get_diagnostics(app_handle, params_val).await,

				_ => unreachable!(),
			}
		},

		"$createTerminal" | "$show" | "$hide" | "$sendText" => {
			// Terminal RPCs via RPC struct
			let rpc_handler = rpc::MainThreadTerminalServiceHandler { app_handle, runtime:rpc_runtime_clone };
			match method {
				"$createTerminal" => rpc_handler.createTerminal(params_val).await,

				"$show" => rpc_handler.show(params_val).await,

				"$hide" => rpc_handler.hide(params_val).await,

				"$sendText" => rpc_handler.sendText(params_val).await,

				_ => unreachable!(),
			}
		},

		"$dispose"
			if params_val
				.as_array()
				.map_or(false, |arr| arr.get(0).and_then(Value::as_u64).is_some())
				&& method == "$dispose" =>
		{
			let rpc_handler = rpc::MainThreadTerminalServiceHandler { app_handle, runtime:rpc_runtime_clone };
			info!("[Track SidecarReq] Assuming '$dispose' is for Terminal due to u64 param (fallback).");
			rpc_handler.dispose(params_val).await
		},

		_ => {
			error!(
				"[Track SidecarReq] Method '{}' from sidecar '{}' has no effect mapping AND no explicit RPC/direct \
				 handler.",
				method, sidecar_id
			);
			Err(error_utils::rpc_error_string(
				format!("Method '{}' not implemented or mapped in Track dispatcher.", method),
				Some("ENOSYS_TRACK"),
			))
		},
	}
}

// Helper function to disambiguate output methods for fallback (simpler version)
fn is_output_method_fallback(method_name:&str) -> bool {
	matches!(method_name, "$register" | "$append" | "$replace" | "$reveal" | "$close")
}

enum EffectCreationError {
	NoEffectMapping,

	ParamParseError(String), // String is already a JSON error string from arg_parse_error_str
}

/// Creates an ActionEffect for commands originating from the frontend.
fn create_effect_for_frontend_command<R:TauriRuntime>(
	_app_handle:&AppHandle<R>,

	_window:&Window<R>,

	command:&str,

	args:Value,
) -> Result<ActionEffect<Arc<AppRuntime>, CommonError, Value>, String> {
	// Errors are JSON strings
	let param_err_fn = |name:&str| -> String { arg_parse_error_str(command, name, "specific type", None) };
	let get_str_arg = |key:&str| {
		args.get(key)
			.and_then(Value::as_str)
			.map(String::from)
			.ok_or_else(|| param_err_fn(key))
	};
	let get_path_arg = |key:&str| get_str_arg(key).map(PathBuf::from);
	let get_i64_arg = |key:&str| args.get(key).and_then(Value::as_i64).ok_or_else(|| param_err_fn(key));
	let get_bool_arg = |key:&str, default_val:bool| args.get(key).and_then(Value::as_bool).unwrap_or(default_val);
	let get_opt_val_arg = |key:&str| args.get(key).cloned();

	trace!("[Track CreateEffect Frontend] Command='{}', Args='{:?}'", command, args);

	match command {
		Land_Echo::REQUEST_READ_FILE => {
			let path = get_path_arg("path")?;
			let effect = fs_effects::read_file(path);
			// Wrap effect to return base64 encoded string
			Ok(ActionEffect::new(Arc::new(move |env_accessor| {
				let effect_clone = effect.clone();
				Box::pin(async move {
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
				get_bool_arg("create", true),
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
				.ok_or_else(|| param_err_fn(&format!("parent of oldPath '{}'", old_path.display())))?;
			Ok(fs_effects::rename(
				old_path,
				parent.join(new_name),
				get_bool_arg("overwrite", false),
			))
		},

		Land_Echo::REQUEST_COPY_PATH => {
			Ok(fs_effects::copy(
				get_path_arg("sourcePath")?,
				get_path_arg("targetParentDir")?.join(get_str_arg("newName")?),
				get_bool_arg("overwrite", false),
			))
		},

		Land_Echo::REQUEST_SAVE_FILE => {
			Ok(documents_effects::try_save(
				Url::parse(&get_str_arg("uri")?).map_err(|e| param_err_fn(&format!("uri parse error: {}", e)))?,
			))
		},

		Land_Echo::REQUEST_SAVE_FILE_AS => {
			Ok(documents_effects::try_save_as(
				Url::parse(&get_str_arg("originalUri")?)
					.map_err(|e| param_err_fn(&format!("originalUri parse error: {}", e)))?,
				args.get("newTargetUri")
					.and_then(Value::as_str)
					.map(|s| Url::parse(s))
					.transpose()
					.map_err(|e| param_err_fn(&format!("newTargetUri parse error: {}", e)))?,
			))
		},

		Land_Echo::REQUEST_APPLY_EDITOR_CHANGES => {
			Ok(documents_effects::apply_changes(
				Url::parse(&get_str_arg("uri")?).map_err(|e| param_err_fn(&format!("uri parse error: {}", e)))?,
				get_i64_arg("versionId")?,
				get_opt_val_arg("changes").ok_or_else(|| param_err_fn("changes"))?,
				get_bool_arg("isDirty", true),
				get_bool_arg("isUndoing", false),
				get_bool_arg("isRedoing", false),
			))
		},

		Land_Echo::REQUEST_OPEN_FILE => {
			Ok(documents_effects::try_open(
				get_opt_val_arg("uriComponents").ok_or_else(|| param_err_fn("uriComponents"))?,
				args.get("languageId").and_then(Value::as_str).map(String::from),
				args.get("content").and_then(Value::as_str).map(String::from),
			))
		},

		Land_Echo::REQUEST_PROXY_EXT_HOST_CALL => {
			Ok(ipc_effects::proxy_call_to_sidecar(
				"cocoon-main".to_string(),
				get_opt_val_arg("callData").ok_or_else(|| param_err_fn("callData"))?,
			))
		},

		Land_Echo::REQUEST_ESTABLISH_HOST_CONNECTION => {
			Ok(ipc_effects::establish_host_connection("cocoon-main".to_string()))
		},

		Land_Echo::REQUEST_WS_SEND | Land_Echo::REQUEST_WS_CONNECT => {
			Err(error_utils::rpc_error_string(
				format!("WebSocket command '{}' not implemented via effects yet.", command),
				Some("ENOSYS_WS"),
			))
		},

		_ => {
			Err(error_utils::rpc_error_string(
				format!("Unknown frontend command for effect creation: {}", command),
				Some("ENOSYS_CMD"),
			))
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
	let get_req_param = |idx:usize, name:&str| params.get(idx).cloned().ok_or_else(|| param_err_fn(name, idx));

	trace!(
		"[Track CreateEffect Sidecar] Method='{}', NumParams={}, Sidecar='{}'",
		method,
		params.len(),
		sidecar_id
	);

	// Helper to wrap effects returning u32 (handles) into Value-returning effects
	let lang_feat_reg_effect = |effect_u32: ActionEffect<Arc<AppRuntime>, CommonError, u32>|
        -> Result<ActionEffect<Arc<AppRuntime>, CommonError, Value>, EffectCreationError> {

        Ok(ActionEffect::new(Arc::new(move |env_accessor| {

            let effect_clone = effect_u32.clone();
            Box::pin(async move {

                env_accessor.run(effect_clone).await.map(Value::from) // CORRECTED and idiomatic
                // or even more concisely:
                // env_accessor.run(effect_clone).await.map(Into::into)
            })
        })))
    };

	// Helper for effects returning ()
	let lang_feat_void_effect = |effect_void: ActionEffect<Arc<AppRuntime>, CommonError, ()>|
    -> Result<ActionEffect<Arc<AppRuntime>, CommonError, Value>, EffectCreationError> {

        Ok(ActionEffect::new(Arc::new(move |env_accessor| {

            let effect_clone = effect_void.clone();
            Box::pin(async move { env_accessor.run(effect_clone).await.map(|_| Value::Null) })
        })))
    };

	match method {
		// --- Configuration ---
		"$getConfiguration" => {
			Ok(config_effects::get_configuration(
				params.get(0).and_then(Value::as_str).map(String::from),
				get_opt_param(1).unwrap_or(Value::Null),
				params.get(2).and_then(Value::as_bool),
			))
		},

		"$updateConfigurationOption" => {
			Ok(config_effects::update_configuration(
				get_str_param(1, "key")?,
				get_req_param(2, "value")?,
				get_opt_param(0)
					.and_then(|v| v.as_u64())
					.ok_or_else(|| param_err_fn("target", 0))? as u32,
				get_opt_param(3).unwrap_or(Value::Null),
				params.get(4).and_then(Value::as_bool),
			))
		},

		"$removeConfigurationOption" => {
			Ok(config_effects::update_configuration(
				get_str_param(1, "key")?,
				Value::Null,
				get_opt_param(0)
					.and_then(|v| v.as_u64())
					.ok_or_else(|| param_err_fn("target", 0))? as u32,
				get_opt_param(2).unwrap_or(Value::Null),
				params.get(3).and_then(Value::as_bool),
			))
		},

		"$inspect" => Ok(config_effects::inspect_configuration(get_str_param(0, "key")?)),

		// --- Workspace Info ---
		"$getWorkspaceFolders" => Ok(workspace_effects::get_workspace_folders()),

		"$requestWorkspaceTrust" => Ok(workspace_effects::request_trust(get_opt_param(0))),

		// --- Storage & Secrets ---
		"$getValue" => {
			Ok(storage_effects::get_storage_item(get_req_param(
				0,
				"target object {scope, key}",
			)?))
		},

		"$setValue" => {
			Ok(storage_effects::set_storage_item(
				get_req_param(0, "target object {scope, key}")?,
				get_req_param(1, "value")?,
			))
		},

		"$getPassword" => {
			Ok(secrets_effects::get_secret(
				get_str_param(0, "extensionId")?,
				get_str_param(1, "key")?,
			))
		},

		"$setPassword" => {
			Ok(secrets_effects::store_secret(
				get_str_param(0, "extensionId")?,
				get_str_param(1, "key")?,
				get_str_param(2, "value")?,
			))
		},

		"$deletePassword" => {
			Ok(secrets_effects::delete_secret(
				get_str_param(0, "extensionId")?,
				get_str_param(1, "key")?,
			))
		},

		// --- Language Features Registration ---
		// Cocoon sends params for $register... as: [0: cocoon_handle, 1: selector_dto, 2: options_dto?, 3:
		// extensionId_dto?] Our effects take: selector, sidecar_id, options_dto. The cocoon_handle (param[0]) is
		// ignored by these effect creators.
		"$registerHoverProvider" => {
			lang_feat_reg_effect(language_feature_effects::register_hover_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerCompletionItemProvider" | "$registerCompletionsProvider" => {
			lang_feat_reg_effect(language_feature_effects::register_completion_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerDefinitionProvider" | "$registerDefinitionSupport" => {
			lang_feat_reg_effect(language_feature_effects::register_definition_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerDeclarationProvider" | "$registerDeclarationSupport" => {
			lang_feat_reg_effect(language_feature_effects::register_declaration_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerImplementationProvider" | "$registerImplementationSupport" => {
			lang_feat_reg_effect(language_feature_effects::register_implementation_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerTypeDefinitionProvider" | "$registerTypeDefinitionSupport" => {
			lang_feat_reg_effect(language_feature_effects::register_type_definition_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerReferencesProvider" | "$registerReferencesSupport" => {
			lang_feat_reg_effect(language_feature_effects::register_references_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerDocumentHighlightProvider" => {
			lang_feat_reg_effect(language_feature_effects::register_document_highlight_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerDocumentSymbolProvider" | "$registerDocumentSymbolSupport" => {
			lang_feat_reg_effect(language_feature_effects::register_document_symbol_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerWorkspaceSymbolProvider" | "$registerWorkspaceSymbolSupport" => {
			lang_feat_reg_effect(language_feature_effects::register_workspace_symbol_provider(
				sidecar_id.to_string(),
				get_opt_param(1), // Selector is actually options for WS provider
			))
		},

		"$registerCodeActionProvider" | "$registerCodeActionSupport" => {
			lang_feat_reg_effect(language_feature_effects::register_code_action_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerCodeLensProvider" | "$registerCodeLensSupport" => {
			lang_feat_reg_effect(language_feature_effects::register_code_lens_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerDocumentFormattingEditProvider" | "$registerDocumentFormattingSupport" => {
			lang_feat_reg_effect(language_feature_effects::register_document_formatting_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerDocumentRangeFormattingEditProvider" | "$registerDocumentRangeFormattingSupport" => {
			lang_feat_reg_effect(language_feature_effects::register_document_range_formatting_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerOnTypeFormattingEditProvider" => {
			lang_feat_reg_effect(language_feature_effects::register_on_type_formatting_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_req_param(2, "onTypeFormattingOptionsDto")?,
			))
		},

		"$registerRenameProvider" | "$registerRenameSupport" => {
			lang_feat_reg_effect(language_feature_effects::register_rename_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerDocumentLinkProvider" | "$registerDocumentLinkSupport" => {
			lang_feat_reg_effect(language_feature_effects::register_document_link_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerDocumentColorProvider" => {
			lang_feat_reg_effect(language_feature_effects::register_document_color_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerFoldingRangeProvider" | "$registerFoldingRangeSupport" => {
			lang_feat_reg_effect(language_feature_effects::register_folding_range_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerSelectionRangeProvider" | "$registerSelectionRangeSupport" => {
			lang_feat_reg_effect(language_feature_effects::register_selection_range_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerCallHierarchyProvider" | "$registerCallHierarchySupport" => {
			lang_feat_reg_effect(language_feature_effects::register_call_hierarchy_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerTypeHierarchyProvider" | "$registerTypeHierarchySupport" => {
			lang_feat_reg_effect(language_feature_effects::register_type_hierarchy_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerLinkedEditingRangeProvider" => {
			lang_feat_reg_effect(language_feature_effects::register_linked_editing_range_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$registerInlayHintsProvider" => {
			lang_feat_reg_effect(language_feature_effects::register_inlay_hints_provider(
				get_req_param(1, "selector")?,
				sidecar_id.to_string(),
				get_opt_param(2),
			))
		},

		"$unregister" | "$unregisterProvider" => {
			lang_feat_void_effect(language_feature_effects::unregister_provider(get_u32_param(
				0,
				"handle (mountain)",
			)?))
		},

		// --- Diagnostics ---
		"$clear" if method == "$clear" && params.len() == 1 && params.get(0).map_or(false, Value::is_string) => {
			Ok(diagnostics_effects::clear_owner_diagnostics(get_str_param(0, "owner")?))
		},

		// --- Methods explicitly routed to RPC handlers or direct handlers (NoEffectMapping) ---
		_ => Err(EffectCreationError::NoEffectMapping),
	}
}
