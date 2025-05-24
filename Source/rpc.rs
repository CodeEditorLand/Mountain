// ---------------------------------------------------------------------------------------------
// Mountain RPC Handlers (rpc.rs)
// --------------------------------------------------------------------------------------------
// Defines the server-side RPC interface that Mountain exposes *to* the Cocoon
// sidecar. It mirrors the `MainThread...Shape` interfaces defined in VS Code's
// `extHost.protocol.ts`.
//
// These handlers are invoked by the `Track` dispatcher, typically as a fallback
// if a direct effect mapping isn't found in `track.rs`.
// For common operations (configuration, storage, secrets, language feature
// registrations), `track.rs` now creates effects directly, so the corresponding
// methods in handlers like `MainThreadConfigurationHandler` or
// `MainThreadLanguageFeaturesHandler` are stubs indicating they should not be
// reached if `track.rs` is correctly routing.
//
// Responsibilities:
// - Defining handler structs (e.g., `MainThreadCommandsHandler`) with necessary
//   context.
// - Implementing `async fn methodName(&self, args: Value)` methods for RPCs not
//   covered by direct effect creation in Track.
// - Parsing `serde_json::Value` arguments received from Cocoon.
// - Calling `handlers::*` functions or, less commonly now for simple
//   operations, creating and dispatching `ActionEffect`s via the `AppRuntime`.
// - Providing the implementation for `vscode.workspace.fs` via
//   `MainThreadFileSystemApiHandler`.
// - Returning `Ok(Value)` or structured error strings using shared error
//   utilities.
//
// Key Interactions:
// - Invoked by `track::dispatch_sidecar_request`.
// - Uses `AppRuntime` (from `self.runtime`) if executing effects (e.g., for
//   dialogs, messages).
// - Calls functions in `handlers::*` (e.g. `handlers::commands::*`,

//   `handlers::workspace::*`).
// - `extHost.protocol.ts` is the contract for method names and DTOs.
// - Uses `handlers::error_utils` for consistent error formatting.
// --------------------------------------------------------------------------------------------

use std::{path::PathBuf, sync::Arc};

// Land_Common imports primarily for types used in effect results or specific DTOs
// and core functionalities like FsReader/FsWriter.
use Land_Common::{
	// Used by error_utils::map_common_error_to_rpc_string
	errors::CommonError,

	// For MainThreadFileSystemApiHandler
	fs_effects::{FsReader, FsWriter},

	// language_feature_effects are now primarily created in track.rs
	// For Dialogs/Messages if handled here
	ui_effects::{self, MessageSeverity, OpenDialogOptions, SaveDialogOptions},
};
use log::{debug, error, info, trace, warn};
// For DTOs if any are deserialized here (e.g. IConfigurationOverrides)
use serde::Deserialize;
use serde_json::{Value, json};
// Window for MainThreadCommands
use tauri::{AppHandle, Manager, Runtime as TauriRuntime, Window, Wry};

use crate::{
	// Though not directly used as State<T> here, kept for broader context
	app_state::AppState,

	// For direct handler calls & error_utils
	handlers,

	// Centralized RPC error utilities
	handlers::error_utils,

	runtime::AppRuntime,
	// Not directly used by RPC methods themselves, but by Track/handlers
	// vine,
};

// Helper to convert path to UriComponents JSON Value for dialog responses
fn path_to_uri_components_value(path:&PathBuf) -> Value {
	let uri_str = url::Url::from_file_path(path).map(|url| url.to_string()).unwrap_or_else(|_| {
		warn!(
			"[RPC Helper] Failed to create file URL from path: {}. Using lossy string.",
			path.display()
		);

		format!("file:///{}", path.to_string_lossy().replace('\\', "/"))
	});

	json!({"$mid": 1, "scheme": "file", "path": path.to_str().unwrap_or(""), "external": uri_str, "fsPath": path.to_str().unwrap_or("") })
}

// --- MainThread Handler Structs ---
// All handlers should have AppHandle and Arc<AppRuntime> for consistency.
#[derive(Clone)]
pub struct MainThreadCommandsHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

#[derive(Clone)]
pub struct MainThreadWorkspaceHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

#[derive(Clone)]
pub struct MainThreadConfigurationHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

#[derive(Clone)]
pub struct MainThreadStorageHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

#[derive(Clone)]
pub struct MainThreadSecretsHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

#[derive(Clone)]
pub struct MainThreadLogHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

#[derive(Clone)]
pub struct MainThreadExtensionServiceHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

// These are mostly placeholders as track.rs calls handlers::* directly
#[derive(Clone)]
pub struct MainThreadOutputServiceHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

#[derive(Clone)]
pub struct MainThreadDiagnosticsHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

#[derive(Clone)]
pub struct MainThreadDocumentsHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

#[derive(Clone)]
pub struct MainThreadLanguageFeaturesHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

#[derive(Clone)]
pub struct MainThreadMessageHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

#[derive(Clone)]
pub struct MainThreadDialogsHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

#[derive(Clone)]
pub struct MainThreadWindowHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

#[derive(Clone)]
pub struct MainThreadStatusBarHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

#[derive(Clone)]
pub struct MainThreadFileSystemApiHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

#[derive(Clone)]
pub struct MainThreadTerminalServiceHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

// --- Method Implementations ---

impl MainThreadCommandsHandler {
	pub async fn executeCommand(&self, args:Value) -> Result<Value, String> {
		debug!(
			"[RPC Cmds] <= $executeCommand: '{}...'",
			args.to_string().chars().take(100).collect::<String>()
		);

		let window = self
			.app_handle
			.get_window("main")
			.ok_or_else(|| error_utils::rpc_error_string("Main window not found".to_string(), Some("ENOWINDOW")))?;

		let args_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$executeCommand", "args array", "array", None))?;

		let command_id = args_array
			.get(0)
			.and_then(Value::as_str)
			.ok_or_else(|| error_utils::rpc_param_error_string("$executeCommand", "commandId", "string", Some(0)))?
			.to_string();

		let command_params_array = args_array.get(1..).map_or_else(Vec::new, |s| s.to_vec());

		let handler_params_obj = json!({ "id": command_id, "args": command_params_array });

		handlers::commands::handle_execute_command(
			self.app_handle.clone(),
			window,
			self.runtime.clone(),
			handler_params_obj,
		)
		.await
	}

	pub async fn getCommands(&self, _args:Value) -> Result<Value, String> {
		debug!("[RPC Cmds] <= $getCommands");

		handlers::commands::handle_get_commands(self.app_handle.clone(), self.runtime.clone()).await
	}

	pub async fn registerCommand(&self, args:Value) -> Result<Value, String> {
		let args_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$registerCommand", "args array", "array", None))?;

		let id = args_array
			.get(0)
			.and_then(Value::as_str)
			.ok_or_else(|| error_utils::rpc_param_error_string("$registerCommand", "id", "string", Some(0)))?
			.to_string();

		info!("[RPC Cmds] <= $registerCommand: id={}", id);

		handlers::commands::handle_register_command(
			self.app_handle.clone(),
			"cocoon-main".to_string(),
			json!({ "id": id }),
		)
		.await
	}

	pub async fn unregisterCommand(&self, args:Value) -> Result<Value, String> {
		let args_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$unregisterCommand", "args array", "array", None))?;

		let id = args_array
			.get(0)
			.and_then(Value::as_str)
			.ok_or_else(|| error_utils::rpc_param_error_string("$unregisterCommand", "id", "string", Some(0)))?
			.to_string();

		info!("[RPC Cmds] <= $unregisterCommand: id={}", id);

		handlers::commands::handle_unregister_command(
			self.app_handle.clone(),
			"cocoon-main".to_string(),
			json!({ "id": id }),
		)
		.await
	}
}

impl MainThreadWorkspaceHandler {
	pub async fn resolveWorkspaceFolder(&self, args:Value) -> Result<Value, String> {
		let uri_components_val = args.as_array().and_then(|a| a.get(0)).cloned().ok_or_else(|| {
			error_utils::rpc_param_error_string("$resolveWorkspaceFolder", "uriComponents", "Value", Some(0))
		})?;

		info!("[RPC Ws] <= $resolveWorkspaceFolder: {:?}", uri_components_val.get("external"));

		handlers::workspace::handle_get_workspace_folder(self.app_handle.clone(), uri_components_val).await
	}

	pub async fn findFiles(&self, args:Value) -> Result<Value, String> {
		debug!(
			"[RPC Ws] <= $findFiles: '{}...'",
			args.to_string().chars().take(100).collect::<String>()
		);

		handlers::workspace::handle_find_files(self.app_handle.clone(), args).await
	}

	// $getWorkspaceFolders, $requestWorkspaceTrust are effects created in track.rs
}

// Helper for RPC method stubs that should now be effects created by track.rs
fn rpc_method_should_be_effect(method_name:&str, args:Value) -> Result<Value, String> {
	warn!(
		"[RPC Handler] {} called (should be an effect created by Track). Args: {:?}",
		method_name, args
	);

	Err(error_utils::rpc_error_string(
		format!("{} should be handled by an effect created in Track.", method_name),
		Some("ENOSYS_EFFECT_FALLBACK"),
	))
}

impl MainThreadConfigurationHandler {
	pub async fn getConfiguration(&self, args:Value) -> Result<Value, String> {
		rpc_method_should_be_effect("$getConfiguration", args)
	}

	pub async fn updateConfigurationOption(&self, args:Value) -> Result<Value, String> {
		rpc_method_should_be_effect("$updateConfigurationOption", args)
	}

	pub async fn removeConfigurationOption(&self, args:Value) -> Result<Value, String> {
		rpc_method_should_be_effect("$removeConfigurationOption", args)
	}

	pub async fn inspect(&self, args:Value) -> Result<Value, String> { rpc_method_should_be_effect("$inspect", args) }
}

impl MainThreadStorageHandler {
	pub async fn getValue(&self, args:Value) -> Result<Value, String> { rpc_method_should_be_effect("$getValue", args) }

	pub async fn setValue(&self, args:Value) -> Result<Value, String> { rpc_method_should_be_effect("$setValue", args) }
}

impl MainThreadSecretsHandler {
	pub async fn getPassword(&self, args:Value) -> Result<Value, String> {
		rpc_method_should_be_effect("$getPassword", args)
	}

	pub async fn setPassword(&self, args:Value) -> Result<Value, String> {
		rpc_method_should_be_effect("$setPassword", args)
	}

	pub async fn deletePassword(&self, args:Value) -> Result<Value, String> {
		rpc_method_should_be_effect("$deletePassword", args)
	}
}

impl MainThreadLogHandler {
	pub async fn log(&self, args:Value) -> Result<Value, String> {
		let args_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$log", "args array", "array", None))?;

		// VS Code LogLevel: Info = 2
		let level_num = args_array.get(0).and_then(Value::as_u64).unwrap_or(2);

		let message_val = args_array.get(1);

		let message_str = match message_val {
			Some(Value::String(s)) => s.clone(),

			Some(Value::Array(arr)) => arr.iter().filter_map(Value::as_str).collect::<Vec<&str>>().join(" "),

			_ => "".to_string(),
		};

		match level_num {
			0 => trace!("[Cocoon EH Log] {}", message_str),

			1 => debug!("[Cocoon EH Log] {}", message_str),

			2 => info!("[Cocoon EH Log] {}", message_str),

			3 => warn!("[Cocoon EH Log] {}", message_str),

			// VS Code Error/Critical
			4 | 5 => error!("[Cocoon EH Log] {}", message_str),

			_ => info!("[Cocoon EH Log] (Unknown Level {}) {}", level_num, message_str),
		}

		Ok(Value::Null)
	}
}

impl MainThreadExtensionServiceHandler {
	// Notifications handled by handlers::extension_status via track.rs dispatcher.
	// These are stubs in case track.rs accidentally routes them here.
	pub async fn onWillActivateExtension(&self, args:Value) -> Result<Value, String> {
		warn!(
			"[RPC ExtSvc] $onWillActivateExtension called (should be direct notification). Args: {:?}",
			args
		);

		Ok(Value::Null)
	}

	pub async fn onDidActivateExtension(&self, args:Value) -> Result<Value, String> {
		warn!(
			"[RPC ExtSvc] $onDidActivateExtension called (should be direct notification). Args: {:?}",
			args
		);

		Ok(Value::Null)
	}

	pub async fn onExtensionActivationError(&self, args:Value) -> Result<Value, String> {
		warn!(
			"[RPC ExtSvc] $onExtensionActivationError called (should be direct notification). Args: {:?}",
			args
		);

		Ok(Value::Null)
	}

	pub async fn onExtensionRuntimeError(&self, args:Value) -> Result<Value, String> {
		warn!(
			"[RPC ExtSvc] $onExtensionRuntimeError called (should be direct notification). Args: {:?}",
			args
		);

		Ok(Value::Null)
	}
}

// Stubs as track.rs calls handlers::* directly or creates effects.
impl MainThreadOutputServiceHandler {
	// All methods are direct handlers in handlers::output.rs via track.rs
}

impl MainThreadDiagnosticsHandler {
	// Methods are direct handlers in handlers::diagnostics.rs or effects via
	// track.rs
}

impl MainThreadDocumentsHandler {
	// Methods are direct handlers in handlers::documents.rs via track.rs
}

impl MainThreadLanguageFeaturesHandler {
	// All common $register...Provider methods are now effects created in track.rs.
	// This method is a catch-all stub for any registration that might unexpectedly
	// fall through. Note: Specific provider types ($registerHoverProvider, etc.)
	// are not explicitly listed here as individual stubs anymore, relying on
	// track.rs to map them to effects. If a specific $register... method is NOT an
	// effect and needs an RPC impl, it would be added here.
	pub async fn CatchAllRegisterProvider(&self, method_name:&str, args:Value) -> Result<Value, String> {
		warn!(
			"[RPC LangFeat] {} called (should be an effect created by Track). Args: {:?}",
			method_name, args
		);

		Err(error_utils::rpc_error_string(
			format!("{} should be an effect created in Track.", method_name),
			Some("ENOSYS_EFFECT_FALLBACK"),
		))
	}

	pub async fn unregister(&self, args:Value) -> Result<Value, String> {
		// $unregister is an effect
		warn!("[RPC LangFeat] $unregister called (should be an effect). Args: {:?}", args);

		Err(error_utils::rpc_error_string(
			"$unregister for language features should be an effect.".to_string(),
			Some("ENOSYS_EFFECT_FALLBACK"),
		))
	}
}

impl MainThreadMessageHandler {
	pub async fn showMessage(&self, args:Value) -> Result<Value, String> {
		let params_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$showMessage", "args array", "array", None))?;

		let severity_num = params_array
			.get(0)
			.and_then(Value::as_u64)
			.ok_or_else(|| error_utils::rpc_param_error_string("$showMessage", "severity", "u64", Some(0)))?;

		let message = params_array
			.get(1)
			.and_then(Value::as_str)
			.ok_or_else(|| error_utils::rpc_param_error_string("$showMessage", "message", "string", Some(1)))?
			.to_string();

		let options_val = params_array.get(2).cloned();

		info!(
			"[RPC MsgSvc] <= $showMessage: severity={}, msg_len={}",
			severity_num,
			message.len()
		);

		let severity_effect = match severity_num {
			1 => MessageSeverity::Info,

			2 => MessageSeverity::Warning,

			3 => MessageSeverity::Error,

			s => {
				warn!("[RPC MsgSvc] Unknown severity {}, defaulting to Info.", s);

				MessageSeverity::Info
			},
		};

		let effect = ui_effects::show_message(severity_effect, message, options_val);

		self.runtime
			.run(effect)
			.await
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "$showMessage"))
	}
}

impl MainThreadDialogsHandler {
	pub async fn showOpenDialog(&self, args:Value) -> Result<Value, String> {
		let params_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$showOpenDialog", "args array", "array", None))?;

		let options_dto_val = params_array.get(0).cloned();

		info!("[RPC Dialogs] <= $showOpenDialog (executing via effect): {:?}", options_dto_val);

		let options_deserialized:Option<OpenDialogOptions> = options_dto_val
			.map(serde_json::from_value)
			.transpose()
			.map_err(|e| error_utils::rpc_error_string(format!("Invalid OpenDialogOptions: {}", e), Some("EBADARG")))?;

		let effect = ui_effects::show_open_dialog(options_deserialized);

		self.runtime
			.run(effect)
			.await
			.map(|paths_opt| {
				json!(
					paths_opt
						.map(|paths| paths.into_iter().map(|p| path_to_uri_components_value(&p)).collect::<Vec<_>>())
				)
			})
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "$showOpenDialog"))
	}

	pub async fn showSaveDialog(&self, args:Value) -> Result<Value, String> {
		let params_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$showSaveDialog", "args array", "array", None))?;

		let options_dto_val = params_array.get(0).cloned();

		info!("[RPC Dialogs] <= $showSaveDialog (executing via effect): {:?}", options_dto_val);

		let options_deserialized:Option<SaveDialogOptions> = options_dto_val
			.map(serde_json::from_value)
			.transpose()
			.map_err(|e| error_utils::rpc_error_string(format!("Invalid SaveDialogOptions: {}", e), Some("EBADARG")))?;

		let effect = ui_effects::show_save_dialog(options_deserialized);

		self.runtime
			.run(effect)
			.await
			.map(|path_opt| json!(path_opt.map(|p| path_to_uri_components_value(&p))))
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "$showSaveDialog"))
	}
}

impl MainThreadWindowHandler {
	pub async fn focusWindow(&self, _args:Value) -> Result<Value, String> {
		info!("[RPC Window] <= $focusWindow");

		if let Some(window) = self.app_handle.get_window("main") {
			window.set_focus().map_err(|e| {
				error_utils::rpc_error_string(format!("Failed to focus window: {}", e), Some("EWINDOW"))
			})?;

			Ok(Value::Null)
		} else {
			Err(error_utils::rpc_error_string(
				"Main window not found".to_string(),
				Some("ENOWINDOW"),
			))
		}
	}

	// TODO: $openUri, $asExternalUri if they are to be RPCs and not direct effects
	// from Track.
}

impl MainThreadStatusBarHandler {
	pub async fn setEntry(&self, args:Value) -> Result<Value, String> {
		let params_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$setEntry", "args array", "array", None))?;

		let id = params_array
			.get(0)
			.and_then(Value::as_str)
			.ok_or_else(|| error_utils::rpc_param_error_string("$setEntry", "id", "string", Some(0)))?;

		// VS Code protocol for IStatusbarEntryDto is complex. `args` here is the whole
		// DTO from Cocoon.
		info!("[RPC StatusBar] <= $setEntry: id='{}'", id);

		trace!("[RPC StatusBar] $setEntry full args: {:?}", args);

		if let Err(e) = self.app_handle.emit_all("mountain://statusbar/set", args.clone()) {
			// Pass full DTO
			error!("[RPC StatusBar] Failed to emit statusbar/set event for {}: {}", id, e);
		}

		Ok(Value::Null)
	}

	pub async fn disposeEntry(&self, args:Value) -> Result<Value, String> {
		let params_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$disposeEntry", "args array", "array", None))?;

		let id = params_array
			.get(0)
			.and_then(Value::as_str)
			.ok_or_else(|| error_utils::rpc_param_error_string("$disposeEntry", "id", "string", Some(0)))?;

		info!("[RPC StatusBar] <= $disposeEntry: id='{}'", id);

		if let Err(e) = self.app_handle.emit_all("mountain://statusbar/dispose", json!({ "id": id })) {
			error!("[RPC StatusBar] Failed to emit statusbar/dispose event for {}: {}", id, e);
		}

		Ok(Value::Null)
	}
}

impl MainThreadTerminalServiceHandler {
	// These now call the more complete handlers in handlers::terminal
	pub async fn createTerminal(&self, params_val:Value) -> Result<Value, String> {
		// params_val for $createTerminal is the options object, not an array
		handlers::terminal::handle_create_terminal(self.app_handle.clone(), params_val).await
	}

	pub async fn show(&self, params_val:Value) -> Result<Value, String> {
		handlers::terminal::handle_show(self.app_handle.clone(), params_val).await
	}

	pub async fn hide(&self, params_val:Value) -> Result<Value, String> {
		handlers::terminal::handle_hide(self.app_handle.clone(), params_val).await
	}

	pub async fn sendText(&self, params_val:Value) -> Result<Value, String> {
		handlers::terminal::handle_send_text(self.app_handle.clone(), params_val).await
	}

	pub async fn dispose(&self, params_val:Value) -> Result<Value, String> {
		handlers::terminal::handle_dispose(self.app_handle.clone(), params_val).await
	}
}

// Nested module for MainThreadFileSystemApiHandler's helper
mod fs_api_helpers {

	use super::{PathBuf, Value, error_utils};

	pub fn path_from_uri_components_for_fs_api(uri_val:&Value) -> Result<PathBuf, String> {
		let scheme = uri_val.get("scheme").and_then(Value::as_str).unwrap_or("file");

		match scheme {
			"file" | "" => {
				Ok(PathBuf::from(uri_val.get("path").and_then(Value::as_str).ok_or_else(|| {
					error_utils::rpc_error_string(
						"Missing 'path' in URI components for FS API".to_string(),
						Some("EBADARG_PATH"),
					)
				})?))
			},

			_ => {
				Err(error_utils::rpc_error_string(
					format!("WorkspaceFS API currently only supports 'file' scheme, got '{}'", scheme),
					Some("ENOTSUP_SCHEME"),
				))
			},
		}
	}
}

impl MainThreadFileSystemApiHandler {
	// Methods are Rust-idiomatic, called by track.rs mapping from
	// workspacefs_$methodName All these methods parse array params:
	// [uri_components, options_or_content]
	pub async fn stat(&self, params_val:Value) -> Result<Value, String> {
		let uri_components = params_val.as_array().and_then(|a| a.get(0)).cloned().ok_or_else(|| {
			error_utils::rpc_param_error_string("workspacefs_stat", "uriComponents", "Value", Some(0))
		})?;

		let path = fs_api_helpers::path_from_uri_components_for_fs_api(&uri_components)?;

		debug!("[RPC FsApiHandler] -> stat: {}", path.display());

		let fs_reader:Arc<dyn FsReader + Send + Sync> = self.runtime.get_environment().require();

		fs_reader
			.stat_file(&path)
			.await
			.map(|stat_obj| serde_json::to_value(stat_obj).unwrap_or(Value::Null))
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "fs.stat"))
	}

	pub async fn read_directory(&self, params_val:Value) -> Result<Value, String> {
		let uri_components = params_val.as_array().and_then(|a| a.get(0)).cloned().ok_or_else(|| {
			error_utils::rpc_param_error_string("workspacefs_readDirectory", "uriComponents", "Value", Some(0))
		})?;

		let path = fs_api_helpers::path_from_uri_components_for_fs_api(&uri_components)?;

		let fs_reader:Arc<dyn FsReader + Send + Sync> = self.runtime.get_environment().require();

		fs_reader
			.read_directory(&path)
			.await
			.map(|entries| json!(entries))
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "fs.readDirectory"))
	}

	pub async fn read_file(&self, params_val:Value) -> Result<Value, String> {
		let uri_components = params_val.as_array().and_then(|a| a.get(0)).cloned().ok_or_else(|| {
			error_utils::rpc_param_error_string("workspacefs_readFile", "uriComponents", "Value", Some(0))
		})?;

		let path = fs_api_helpers::path_from_uri_components_for_fs_api(&uri_components)?;

		let fs_reader:Arc<dyn FsReader + Send + Sync> = self.runtime.get_environment().require();

		fs_reader
			.read_file(&path)
			.await
			.map(|bytes| json!(base64::encode(&bytes)))
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "fs.readFile"))
	}

	pub async fn write_file(&self, params_val:Value) -> Result<Value, String> {
		let params_array = params_val.as_array().ok_or_else(|| {
			error_utils::rpc_param_error_string("workspacefs_writeFile", "params array", "array", None)
		})?;

		let uri_components = params_array.get(0).cloned().ok_or_else(|| {
			error_utils::rpc_param_error_string("workspacefs_writeFile", "uriComponents", "Value", Some(0))
		})?;

		let content_b64 = params_array.get(1).and_then(Value::as_str).ok_or_else(|| {
			error_utils::rpc_param_error_string("workspacefs_writeFile", "contentBase64", "string", Some(1))
		})?;

		// IFileWriteOptions
		let options_val = params_array.get(2).cloned().unwrap_or(Value::Null);

		// VS Code default for create is true for fs provider
		let create = options_val.get("create").and_then(Value::as_bool).unwrap_or(true);

		// VS Code default for overwrite is false
		let overwrite = options_val.get("overwrite").and_then(Value::as_bool).unwrap_or(false);

		let path = fs_api_helpers::path_from_uri_components_for_fs_api(&uri_components)?;

		let bytes = base64::decode(content_b64)
			.map_err(|e| error_utils::rpc_error_string(format!("Invalid base64 content: {}", e), Some("EBADMSG")))?;

		let fs_writer:Arc<dyn FsWriter + Send + Sync> = self.runtime.get_environment().require();

		fs_writer
			.write_file(&path, bytes, create, overwrite)
			.await
			.map(|_| Value::Null)
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "fs.writeFile"))
	}

	pub async fn create_directory(&self, params_val:Value) -> Result<Value, String> {
		let uri_components = params_val.as_array().and_then(|a| a.get(0)).cloned().ok_or_else(|| {
			error_utils::rpc_param_error_string("workspacefs_createDirectory", "uriComponents", "Value", Some(0))
		})?;

		let path = fs_api_helpers::path_from_uri_components_for_fs_api(&uri_components)?;

		let fs_writer:Arc<dyn FsWriter + Send + Sync> = self.runtime.get_environment().require();

		// vscode.workspace.fs.createDirectory is recursive
		fs_writer
			.create_directory(&path, true)
			.await
			.map(|_| Value::Null)
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "fs.createDirectory"))
	}

	pub async fn delete(&self, params_val:Value) -> Result<Value, String> {
		let params_array = params_val
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("workspacefs_delete", "params array", "array", None))?;

		let uri_components = params_array.get(0).cloned().ok_or_else(|| {
			error_utils::rpc_param_error_string("workspacefs_delete", "uriComponents", "Value", Some(0))
		})?;

		// IFileDeleteOptions
		let options_val = params_array.get(1).cloned().unwrap_or(Value::Null);

		let recursive = options_val.get("recursive").and_then(Value::as_bool).unwrap_or(false);

		let use_trash = options_val.get("useTrash").and_then(Value::as_bool).unwrap_or(false);

		let path = fs_api_helpers::path_from_uri_components_for_fs_api(&uri_components)?;

		let fs_writer:Arc<dyn FsWriter + Send + Sync> = self.runtime.get_environment().require();

		fs_writer
			.delete(&path, recursive, use_trash)
			.await
			.map(|_| Value::Null)
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "fs.delete"))
	}

	pub async fn rename(&self, params_val:Value) -> Result<Value, String> {
		let params_array = params_val
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("workspacefs_rename", "params array", "array", None))?;

		let source_uri_comp = params_array
			.get(0)
			.cloned()
			.ok_or_else(|| error_utils::rpc_param_error_string("workspacefs_rename", "sourceUri", "Value", Some(0)))?;

		let target_uri_comp = params_array
			.get(1)
			.cloned()
			.ok_or_else(|| error_utils::rpc_param_error_string("workspacefs_rename", "targetUri", "Value", Some(1)))?;

		// IFileOverwriteOptions
		let options_val = params_array.get(2).cloned().unwrap_or(Value::Null);

		let overwrite = options_val.get("overwrite").and_then(Value::as_bool).unwrap_or(false);

		let source_path = fs_api_helpers::path_from_uri_components_for_fs_api(&source_uri_comp)?;

		let target_path = fs_api_helpers::path_from_uri_components_for_fs_api(&target_uri_comp)?;

		let fs_writer:Arc<dyn FsWriter + Send + Sync> = self.runtime.get_environment().require();

		fs_writer
			.rename(&source_path, &target_path, overwrite)
			.await
			.map(|_| Value::Null)
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "fs.rename"))
	}

	pub async fn copy(&self, params_val:Value) -> Result<Value, String> {
		let params_array = params_val
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("workspacefs_copy", "params array", "array", None))?;

		let source_uri_comp = params_array
			.get(0)
			.cloned()
			.ok_or_else(|| error_utils::rpc_param_error_string("workspacefs_copy", "sourceUri", "Value", Some(0)))?;

		let target_uri_comp = params_array
			.get(1)
			.cloned()
			.ok_or_else(|| error_utils::rpc_param_error_string("workspacefs_copy", "targetUri", "Value", Some(1)))?;

		// IFileOverwriteOptions
		let options_val = params_array.get(2).cloned().unwrap_or(Value::Null);

		let overwrite = options_val.get("overwrite").and_then(Value::as_bool).unwrap_or(false);

		let source_path = fs_api_helpers::path_from_uri_components_for_fs_api(&source_uri_comp)?;

		let target_path = fs_api_helpers::path_from_uri_components_for_fs_api(&target_uri_comp)?;

		let fs_writer:Arc<dyn FsWriter + Send + Sync> = self.runtime.get_environment().require();

		fs_writer
			.copy(&source_path, &target_path, overwrite)
			.await
			.map(|_| Value::Null)
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "fs.copy"))
	}
}

// --- Setup Function ---
pub fn setup_mountain_rpc_server<R:TauriRuntime>(_app_handle:AppHandle<R>, _runtime:Arc<AppRuntime>) {
	info!("[RPC Setup] Mountain RPC handlers are conceptually available for Track dispatcher.");

	// Track.rs will instantiate these handler structs as needed when an RPC
	// call falls through.
}
