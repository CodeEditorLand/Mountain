// ---------------------------------------------------------------------------------------------
// Mountain RPC Handlers (rpc.rs)
// --------------------------------------------------------------------------------------------
// Defines the server-side RPC interface that Mountain exposes *to* the Cocoon
// sidecar. It mirrors the `MainThread...Shape` interfaces defined in VS Code's
// `extHost.protocol.ts`, providing the methods that Cocoon's shims expect to
// call.
//
// These handlers are invoked by the `Track` dispatcher. They parse incoming
// arguments and either delegate to specific business logic in `handlers/*.rs`
// or, preferably, create and dispatch `ActionEffect`s via the `AppRuntime`.
//
// Responsibilities:
// - Defining handler structs (e.g., `MainThreadCommandsHandler`) with necessary
//   context.
// - Implementing `async fn methodName(&self, args: Value)` methods.
// - Parsing `serde_json::Value` arguments received from Cocoon.
// - Creating and running `ActionEffect`s or calling `handlers::*` functions.
// - Returning `Ok(Value)` or structured error strings.
//
// Key Interactions:
// - Invoked by `track::dispatch_sidecar_request`.
// - Uses `AppRuntime` (from `self.runtime`) to execute effects.
// - Calls functions in `handlers::*`.
// - `extHost.protocol.ts` is the contract for method names and DTOs.
// --------------------------------------------------------------------------------------------

// Ensure PathBuf is in scope for path_from_uri_components_for_fs_api
use std::{path::PathBuf, sync::Arc};

use Land_Common::{
	config_effects::{self, ConfigurationTarget, IConfigurationOverrides},

	diagnostics_effects,

	documents_effects,

	effect::ActionEffect,

	errors::CommonError,

	// Added FsReader, FsWriter for MainThreadFileSystemApiHandler
	fs_effects::{FsReader, FsWriter},

	language_feature_effects,

	output_effects,

	secrets_effects,

	storage_effects,

	// Added specific UI effect types
	ui_effects::{self, MessageSeverity, OpenDialogOptions, SaveDialogOptions},

	workspace_effects,
};
use log::{debug, error, info, trace, warn};
use serde::Deserialize;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime as TauriRuntime, State, Window, Wry};

use crate::{app_state::AppState, handlers, runtime::AppRuntime, vine};

// --- Helper for structured errors ---
fn rpc_error(message:String, code:Option<&str>) -> String {
	json!({ "message": message, "code": code.unwrap_or("EUNKNOWN_RPC") }).to_string()
}

fn rpc_param_error(method_name:&str, param_name:&str, expected_type:&str, idx:Option<usize>) -> String {
	let base_msg = format!(
		"Missing or invalid '{}' parameter (expected {}) for RPC method '{}'",
		param_name, expected_type, method_name
	);

	let full_msg = if let Some(i) = idx {
		format!("{} at arg index {}", base_msg, i)
	} else {
		base_msg
	};

	// Log the error
	error!("[RPC ParamError] {}", full_msg);

	rpc_error(full_msg, Some("EBADARG"))
}

fn map_common_error_to_rpc_string(e:CommonError, operation_context:&str) -> String {
	error!("[RPC Op Error] CommonError during '{}': {}", operation_context, e);

	let code = match e {
		CommonError::FsNotFound(_) => "ENOENT",

		CommonError::FsPermissionDenied(..) => "EACCES",

		CommonError::FsFileExists(_) => "EEXIST",

		CommonError::ConfigUpdate(..) | CommonError::ConfigLoad(_) => "ECONFIG",

		CommonError::InvalidArg(..) => "EBADARG",

		CommonError::NotImplemented(_) => "ENOSYS",

		_ => "EINTERNAL",
	};

	rpc_error(e.to_string(), Some(code))
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
	// Added runtime for consistency, though $log is simple
}

#[derive(Clone)]
pub struct MainThreadExtensionServiceHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

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

// Helper to convert path to UriComponents JSON Value for dialog responses
fn path_to_uri_components_value(path:&PathBuf) -> Value {
	let uri_str = url::Url::from_file_path(path)
		.map(|url| url.to_string())
		.unwrap_or_else(|_| format!("file:///{}", path.to_string_lossy().replace('\\', "/")));

	json!({"scheme": "file", "path": path.to_str().unwrap_or(""), "external": uri_str, "$mid": 1 })
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
			.ok_or_else(|| rpc_error("Main window not found".to_string(), Some("ENOWINDOW")))?;

		// Assuming args is already the {id, args} object from Track's parsing,

		// or if not, needs to be parsed from array as in previous versions.
		// For this merge, assuming `args` IS the DTO {id, args} directly.
		// If `args` is Value::Array([commandId, ...rest]), then previous parsing is
		// needed. Let's stick to the latest version (rpc.rs from 185_MODEL) where
		// Track passes `args` which then is handled by
		// `handlers::commands::handle_execute_command`. The handler expects: json!({

		// "id": command_id, "args": command_args_array }) The incoming `args` from
		// Cocoon for $executeCommand is `[commandId: string, ...args: any[]]`
		// So, we MUST parse it here.
		let args_array = args
			.as_array()
			.ok_or_else(|| rpc_param_error("$executeCommand", "args array", "array", None))?;

		let command_id = args_array
			.get(0)
			.and_then(Value::as_str)
			.ok_or_else(|| rpc_param_error("$executeCommand", "commandId", "string", Some(0)))?
			.to_string();

		let command_params_array = args_array.get(1..).map_or_else(Vec::new, |s| s.to_vec());

		let handler_params = json!({ "id": command_id, "args": command_params_array });

		handlers::commands::handle_execute_command(
			self.app_handle.clone(),
			window,
			self.runtime.clone(),
			handler_params,
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
			.ok_or_else(|| rpc_param_error("$registerCommand", "args array", "array", None))?;

		let id = args_array
			.get(0)
			.and_then(Value::as_str)
			.ok_or_else(|| rpc_param_error("$registerCommand", "id", "string", Some(0)))?
			.to_string();

		info!("[RPC Cmds] <= $registerCommand: id={}", id);

		// TODO: Get from request context
		let sidecar_id = "cocoon-main".to_string();

		handlers::commands::handle_register_command(self.app_handle.clone(), sidecar_id, json!({ "id": id })).await
	}

	pub async fn unregisterCommand(&self, args:Value) -> Result<Value, String> {
		let args_array = args
			.as_array()
			.ok_or_else(|| rpc_param_error("$unregisterCommand", "args array", "array", None))?;

		let id = args_array
			.get(0)
			.and_then(Value::as_str)
			.ok_or_else(|| rpc_param_error("$unregisterCommand", "id", "string", Some(0)))?
			.to_string();

		info!("[RPC Cmds] <= $unregisterCommand: id={}", id);

		let sidecar_id = "cocoon-main".to_string();

		handlers::commands::handle_unregister_command(self.app_handle.clone(), sidecar_id, json!({ "id": id })).await
	}
}

impl MainThreadWorkspaceHandler {
	pub async fn resolveWorkspaceFolder(&self, args:Value) -> Result<Value, String> {
		warn!("[RPC Ws] <= $resolveWorkspaceFolder (STUBBED in handler): {:?}", args);

		// args is Value::Array([uriComponents: Value])
		let uri_components_val = args
			.as_array()
			.and_then(|a| a.get(0))
			.cloned()
			.ok_or_else(|| rpc_param_error("$resolveWorkspaceFolder", "uriComponents", "Value", Some(0)))?;

		handlers::workspace::handle_get_workspace_folder(self.app_handle.clone(), uri_components_val).await
	}

	pub async fn findFiles(&self, args:Value) -> Result<Value, String> {
		debug!(
			"[RPC Ws] <= $findFiles: '{}...'",
			args.to_string().chars().take(100).collect::<String>()
		);

		// args is Value::Array([include, exclude?, options?])
		handlers::workspace::handle_find_files(self.app_handle.clone(), args).await
	}
}

impl MainThreadConfigurationHandler {
	// Most configuration methods are now effects created directly in track.rs.
	// If specific RPC methods were needed here, they would parse args and create
	// effects: Example for $getConfiguration:
	// pub async fn getConfiguration(&self, args: Value) -> Result<Value, String> {

	//     debug!("[RPC Config] <= $getConfiguration: '{}...'",

	// args.to_string().chars().take(100).collect::<String>());

	//     let params_array = args.as_array().ok_or_else(||
	// rpc_param_error("$getConfiguration", "args array", "array", None))?;     let
	// section = params_array.get(0).and_then(Value::as_str).map(String::from);

	//     let overrides_val = params_array.get(1).cloned().unwrap_or(Value::Null);

	// Note: scopeToLanguage might not be present in all calls, handle Option
	//
	//     let scope_to_language = params_array.get(2).and_then(Value::as_bool);

	//     let effect = config_effects::get_configuration(section, overrides_val,

	// scope_to_language);     self.runtime.run(effect).await.map_err(|e|
	// map_common_error_to_rpc_string(e, "$getConfiguration")) }
}

impl MainThreadStorageHandler {
	// $getValue and $setValue are effects created in track.rs.
	// If RPC methods were needed:
	// pub async fn getValue(&self, args: Value) -> Result<Value, String> {

	//     trace!("[RPC Storage] <= $getValue: Args: {:?}", args);

	//     let params_obj = args.as_array().and_then(|a|
	// a.get(0)).cloned().ok_or_else(|| rpc_param_error("$getValue", "params
	// object", "object", Some(0)))?;     let effect =
	// Assuming effect takes the
	// storage_effects::get_storage_item(params_obj);

	// DTO     self.runtime.run(effect).await.map_err(|e|
	// map_common_error_to_rpc_string(e, "$getValue")) }
}

impl MainThreadSecretsHandler {
	// $getPassword, $setPassword, $deletePassword are effects created in track.rs.
}

impl MainThreadLogHandler {
	pub async fn log(&self, args:Value) -> Result<Value, String> {
		let args_array = args
			.as_array()
			.ok_or_else(|| rpc_param_error("$log", "args array", "array", None))?;

		// VS Code LogLevel: Info = 2
		let level_num = args_array.get(0).and_then(Value::as_u64).unwrap_or(2);

		let message_val = args_array.get(1);

		let message_str = match message_val {
			Some(Value::String(s)) => s.clone(),

			Some(Value::Array(arr)) => arr.iter().filter_map(Value::as_str).collect::<Vec<&str>>().join(" "), /* Join string parts if array */
			_ => "".to_string(),
		};

		match level_num {
			// VS Code LogLevel mapping
			// Trace
			0 => trace!("[Cocoon EH Log] {}", message_str),

			// Debug
			1 => debug!("[Cocoon EH Log] {}", message_str),

			// Info
			2 => info!("[Cocoon EH Log] {}", message_str),

			// Warning
			3 => warn!("[Cocoon EH Log] {}", message_str),

			// Error, Critical (map both to error)
			4 | 5 => error!("[Cocoon EH Log] {}", message_str),

			// Fallback
			_ => info!("[Cocoon EH Log] (Unknown Level {}) {}", level_num, message_str),
		}

		Ok(Value::Null)
	}
}

impl MainThreadExtensionServiceHandler {
	// Notifications like $onWillActivateExtension are handled directly by track.rs
	// routing to handlers::extension_status.
}

impl MainThreadOutputServiceHandler {
	// All methods are routed by track.rs directly to handlers::output.
}

impl MainThreadDiagnosticsHandler {
	// All methods are routed by track.rs directly to handlers::diagnostics.
}

impl MainThreadDocumentsHandler {
	// All methods are routed by track.rs directly to handlers::documents.
}

impl MainThreadLanguageFeaturesHandler {
	pub async fn registerHoverProvider(&self, args:Value) -> Result<Value, String> {
		info!("[RPC LangFeat] <= $registerHoverProvider");

		let params_array = args
			.as_array()
			.ok_or_else(|| rpc_param_error("$registerHoverProvider", "args array", "array", None))?;

		let _cocoon_handle = params_array
			.get(0)
			.and_then(Value::as_u64)
			.ok_or_else(|| rpc_param_error("$registerHoverProvider", "cocoon_handle", "u64", Some(0)))?
			as u32;

		let selector = params_array
			.get(1)
			.cloned()
			.ok_or_else(|| rpc_param_error("$registerHoverProvider", "selector", "Value", Some(1)))?;

		let sidecar_id = "cocoon-main".to_string();

		let effect = language_feature_effects::register_hover_provider(selector, sidecar_id);

		self.runtime
			.run(effect)
			.await
			.map(|h_mountain| json!(h_mountain))
			.map_err(|e| map_common_error_to_rpc_string(e, "$registerHoverProvider"))
	}

	pub async fn registerCompletionsProvider(&self, args:Value) -> Result<Value, String> {
		info!("[RPC LangFeat] <= $registerCompletionsProvider");

		let params_array = args
			.as_array()
			.ok_or_else(|| rpc_param_error("$registerCompletionsProvider", "args array", "array", None))?;

		let _cocoon_handle = params_array
			.get(0)
			.and_then(Value::as_u64)
			.ok_or_else(|| rpc_param_error("$registerCompletionsProvider", "cocoon_handle", "u64", Some(0)))?
			as u32;

		let selector = params_array
			.get(1)
			.cloned()
			.ok_or_else(|| rpc_param_error("$registerCompletionsProvider", "selector", "Value", Some(1)))?;

		let trigger_chars_val = params_array
			.get(2)
			.cloned()
			.ok_or_else(|| rpc_param_error("$registerCompletionsProvider", "triggerCharacters", "array", Some(2)))?;

		let trigger_chars:Vec<String> = serde_json::from_value(trigger_chars_val).map_err(|e| {
			rpc_param_error(
				"$registerCompletionsProvider",
				"triggerCharacters",
				&format!("valid string array: {}", e),
				Some(2),
			)
		})?;

		let sidecar_id = "cocoon-main".to_string();

		// options (like supportsResolveDetails) are part of the effect if needed
		let effect = language_feature_effects::register_completion_provider(selector, trigger_chars, sidecar_id);

		self.runtime
			.run(effect)
			.await
			.map(|h_mountain| json!(h_mountain))
			.map_err(|e| map_common_error_to_rpc_string(e, "$registerCompletionsProvider"))
	}

	pub async fn registerDefinitionSupport(&self, args:Value) -> Result<Value, String> {
		info!("[RPC LangFeat] <= $registerDefinitionSupport");

		let params_array = args
			.as_array()
			.ok_or_else(|| rpc_param_error("$registerDefinitionSupport", "args array", "array", None))?;

		let cocoon_handle = params_array
			.get(0)
			.and_then(Value::as_u64)
			.ok_or_else(|| rpc_param_error("$registerDefinitionSupport", "cocoon_handle", "u64", Some(0)))?
			as u32;

		let selector = params_array
			.get(1)
			.cloned()
			.ok_or_else(|| rpc_param_error("$registerDefinitionSupport", "selector", "Value", Some(1)))?;

		let sidecar_id = "cocoon-main".to_string();

		let effect = language_feature_effects::register_definition_provider(selector, sidecar_id, cocoon_handle);

		self.runtime
			.run(effect)
			.await
			.map(|h_mountain| json!(h_mountain))
			.map_err(|e| map_common_error_to_rpc_string(e, "$registerDefinitionSupport"))
	}

	// TODO: Implement ALL other $register<Feature>Provider methods as per
	// extHost.protocol.ts MainThreadLanguageFeaturesShape. Each will typically:
	// 1. Parse args: `cocoon_handle`, `selector`, `extensionIdDto`, and
	//    feature-specific metadata/options.
	// 2. Construct the appropriate
	//    `language_feature_effects::register_XYZ_provider` effect. The effect
	//    itself will then call `environment.register_provider` with all necessary
	//    details.
	// 3. Run the effect and return the Mountain handle.

	pub async fn unregister(&self, args:Value) -> Result<Value, String> {
		info!("[RPC LangFeat] <= $unregister");

		let params_array = args
			.as_array()
			.ok_or_else(|| rpc_param_error("$unregister", "args array", "array", None))?;

		let handle_mountain = params_array
			.get(0)
			.and_then(Value::as_u64)
			.ok_or_else(|| rpc_param_error("$unregister", "handle", "u64", Some(0)))? as u32;

		let effect = language_feature_effects::unregister_provider(handle_mountain);

		self.runtime
			.run(effect)
			.await
			.map(|_| Value::Null)
			.map_err(|e| map_common_error_to_rpc_string(e, "$unregister"))
	}
}

impl MainThreadMessageHandler {
	pub async fn showMessage(&self, args:Value) -> Result<Value, String> {
		let params_array = args
			.as_array()
			.ok_or_else(|| rpc_param_error("$showMessage", "args array", "array", None))?;

		let severity_num = params_array
			.get(0)
			.and_then(Value::as_u64)
			.ok_or_else(|| rpc_param_error("$showMessage", "severity", "u64", Some(0)))?;

		let message = params_array
			.get(1)
			.and_then(Value::as_str)
			.ok_or_else(|| rpc_param_error("$showMessage", "message", "string", Some(1)))?
			.to_string();

		// MainThreadMessageOptions DTO (might include modal, detail, items/commands)
		let options_val = params_array.get(2).cloned();

		info!(
			"[RPC MsgSvc] <= $showMessage: severity={}, msg_len={}",
			severity_num,
			message.len()
		);

		trace!("[RPC MsgSvc] $showMessage options: {:?}", options_val);

		let severity_effect = match severity_num {
			// VS Code Severity enum
			1 => MessageSeverity::Info,

			2 => MessageSeverity::Warning,

			3 => MessageSeverity::Error,

			s => {
				warn!("[RPC MsgSvc] Unknown severity {} from $showMessage, defaulting to Info.", s);

				MessageSeverity::Info
			},
		};

		// The `ui_effects::show_message` needs to deserialize `options_val` into
		// `MessageOptions` if it expects typed options with actions etc.
		let effect = ui_effects::show_message(severity_effect, message, options_val);

		self.runtime
			.run(effect)
			.await
			.map_err(|e| map_common_error_to_rpc_string(e, "$showMessage"))
		// The effect should return Option<String> (selected item title) or
		// Option<u32> (handle) This needs to be mapped back to a Value
		// (likely string or null) for Cocoon. For simple messages,

		// Value::Null is fine. For messages with buttons, the returned ID needs
		// to be wrapped in Value.
	}
}

impl MainThreadDialogsHandler {
	pub async fn showOpenDialog(&self, args:Value) -> Result<Value, String> {
		let params_array = args
			.as_array()
			.ok_or_else(|| rpc_param_error("$showOpenDialog", "args array", "array", None))?;

		let options_dto_val = params_array.get(0).cloned();

		warn!("[RPC Dialogs] <= $showOpenDialog (STUBBED VIA EFFECT): {:?}", options_dto_val);

		let options_deserialized:Option<OpenDialogOptions> = options_dto_val
			.map(serde_json::from_value)
			.transpose()
			.map_err(|e| rpc_error(format!("Invalid OpenDialogOptions: {}", e), Some("EBADARG")))?;

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
			.map_err(|e| map_common_error_to_rpc_string(e, "$showOpenDialog"))
	}

	pub async fn showSaveDialog(&self, args:Value) -> Result<Value, String> {
		let params_array = args
			.as_array()
			.ok_or_else(|| rpc_param_error("$showSaveDialog", "args array", "array", None))?;

		let options_dto_val = params_array.get(0).cloned();

		warn!("[RPC Dialogs] <= $showSaveDialog (STUBBED VIA EFFECT): {:?}", options_dto_val);

		let options_deserialized:Option<SaveDialogOptions> = options_dto_val
			.map(serde_json::from_value)
			.transpose()
			.map_err(|e| rpc_error(format!("Invalid SaveDialogOptions: {}", e), Some("EBADARG")))?;

		let effect = ui_effects::show_save_dialog(options_deserialized);

		self.runtime
			.run(effect)
			.await
			.map(|path_opt| json!(path_opt.map(|p| path_to_uri_components_value(&p))))
			.map_err(|e| map_common_error_to_rpc_string(e, "$showSaveDialog"))
	}
}

impl MainThreadWindowHandler {
	pub async fn focusWindow(&self, _args:Value) -> Result<Value, String> {
		info!("[RPC Window] <= $focusWindow");

		if let Some(window) = self.app_handle.get_window("main") {
			window
				.set_focus()
				.map_err(|e| rpc_error(format!("Failed to focus window: {}", e), Some("EWINDOW")))?;

			Ok(Value::Null)
		} else {
			Err(rpc_error("Main window not found".to_string(), Some("ENOWINDOW")))
		}
	}
}

impl MainThreadStatusBarHandler {
	// $setEntry and $disposeEntry typically involve UI updates via Tauri events.
	// Their implementations in `track.rs` (direct emit) or here are fine.
	// Using effect for this would be overkill unless complex state logic is
	// involved.
	pub async fn setEntry(&self, args:Value) -> Result<Value, String> {
		let params_array = args
			.as_array()
			.ok_or_else(|| rpc_param_error("$setEntry", "args array", "array", None))?;

		let id = params_array
			.get(0)
			.and_then(Value::as_str)
			.ok_or_else(|| rpc_param_error("$setEntry", "id", "string", Some(0)))?;

		let text_val = params_array
			.get(4)
			.cloned()
			 // VS Code protocol param index for text
			.ok_or_else(|| rpc_param_error("$setEntry", "text", "string", Some(4)))?;

		info!(
			"[RPC StatusBar] <= $setEntry: id='{}', text(brief)='{}...'",
			id,
			text_val.as_str().unwrap_or("").chars().take(30).collect::<String>()
		);

		trace!("[RPC StatusBar] $setEntry full args: {:?}", args);

		if let Err(e) = self.app_handle.emit_all("mountain://statusbar/set", args) {
			// Pass full DTO
			error!("[RPC StatusBar] Failed to emit statusbar/set event for {}: {}", id, e);
		}

		Ok(Value::Null)
	}

	pub async fn disposeEntry(&self, args:Value) -> Result<Value, String> {
		let params_array = args
			.as_array()
			.ok_or_else(|| rpc_param_error("$disposeEntry", "args array", "array", None))?;

		let id = params_array
			.get(0)
			.and_then(Value::as_str)
			.ok_or_else(|| rpc_param_error("$disposeEntry", "id", "string", Some(0)))?;

		info!("[RPC StatusBar] <= $disposeEntry: id='{}'", id);

		if let Err(e) = self.app_handle.emit_all("mountain://statusbar/dispose", json!({ "id": id })) {
			error!("[RPC StatusBar] Failed to emit statusbar/dispose event for {}: {}", id, e);
		}

		Ok(Value::Null)
	}
}

// Nested module for MainThreadFileSystemApiHandler's helper to avoid polluting
// rpc.rs top-level.
mod fs_api_helpers {

	// Import necessary items from parent module
	use super::{PathBuf, Value, rpc_error, rpc_param_error};

	pub fn path_from_uri_components_for_fs_api(uri_val:&Value) -> Result<PathBuf, String> {
		let scheme = uri_val.get("scheme").and_then(|v| v.as_str()).unwrap_or("file");

		match scheme {
			"file" | "" => {
				// Allow empty scheme as well for file paths
				let path_str = uri_val.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
					rpc_error("Missing 'path' in URI components for FS API".to_string(), Some("EBADARG"))
				})?;

				Ok(PathBuf::from(path_str))
			},

			_ => {
				Err(rpc_error(
					format!("WorkspaceFS API currently only supports 'file' scheme, got '{}'", scheme),
					Some("ENOTSUP"),
				))
			},
		}
	}
}

impl MainThreadFileSystemApiHandler {
	pub async fn stat(&self, params_val:Value) -> Result<Value, String> {
		// Method names changed to be Rust idiomatic if called internally
		let uri_components = params_val
			.as_array()
			.and_then(|a| a.get(0))
			.cloned()
			.ok_or_else(|| rpc_param_error("workspacefs_$stat", "uriComponents", "Value", Some(0)))?;

		let path = fs_api_helpers::path_from_uri_components_for_fs_api(&uri_components)?;

		debug!("[RPC FsApiHandler] <= $stat: {}", path.display());

		let fs_reader:Arc<dyn FsReader + Send + Sync> = self.runtime.get_environment().require();

		fs_reader
			.stat_file(&path)
			.await
			.map(|stat_obj| serde_json::to_value(stat_obj).unwrap_or(Value::Null))
			.map_err(|e| map_common_error_to_rpc_string(e, "workspacefs_$stat"))
	}

	pub async fn read_directory(&self, params_val:Value) -> Result<Value, String> {
		let uri_components = params_val
			.as_array()
			.and_then(|a| a.get(0))
			.cloned()
			.ok_or_else(|| rpc_param_error("workspacefs_$readDirectory", "uriComponents", "Value", Some(0)))?;

		let path = fs_api_helpers::path_from_uri_components_for_fs_api(&uri_components)?;

		debug!("[RPC FsApiHandler] <= $readDirectory: {}", path.display());

		let fs_reader:Arc<dyn FsReader + Send + Sync> = self.runtime.get_environment().require();

		fs_reader.read_directory(&path).await
             // Assumes entries is Vec<(String, CommonFileType)> which is serializable
			.map(|entries| json!(entries))
            .map_err(|e| map_common_error_to_rpc_string(e, "workspacefs_$readDirectory"))
	}

	pub async fn read_file(&self, params_val:Value) -> Result<Value, String> {
		let uri_components = params_val
			.as_array()
			.and_then(|a| a.get(0))
			.cloned()
			.ok_or_else(|| rpc_param_error("workspacefs_$readFile", "uriComponents", "Value", Some(0)))?;

		let path = fs_api_helpers::path_from_uri_components_for_fs_api(&uri_components)?;

		debug!("[RPC FsApiHandler] <= $readFile: {}", path.display());

		let fs_reader:Arc<dyn FsReader + Send + Sync> = self.runtime.get_environment().require();

		fs_reader.read_file(&path).await
             // Cocoon expects base64 encoded content
			.map(|bytes| json!(base64::encode(&bytes)))
            .map_err(|e| map_common_error_to_rpc_string(e, "workspacefs_$readFile"))
	}

	pub async fn write_file(&self, params_val:Value) -> Result<Value, String> {
		let params_array = params_val
			.as_array()
			.ok_or_else(|| rpc_param_error("workspacefs_$writeFile", "params array", "array", None))?;

		let uri_components = params_array
			.get(0)
			.cloned()
			.ok_or_else(|| rpc_param_error("workspacefs_$writeFile", "uriComponents", "Value", Some(0)))?;

		let content_b64 = params_array
			.get(1)
			.and_then(Value::as_str)
			.ok_or_else(|| rpc_param_error("workspacefs_$writeFile", "contentBase64", "string", Some(1)))?;

		// IFileWriteOptions: { create: boolean, overwrite: boolean, unlock?: boolean,

		// atomic?: { rangesOffset?: number; } | boolean; }

		let options_val = params_array.get(2).cloned().unwrap_or(Value::Null);

		// Default to true for create
		let create = options_val.get("create").and_then(Value::as_bool).unwrap_or(true);

		// Default to false for overwrite (VS Code default)
		let overwrite = options_val.get("overwrite").and_then(Value::as_bool).unwrap_or(false);

		let path = fs_api_helpers::path_from_uri_components_for_fs_api(&uri_components)?;

		debug!("[RPC FsApiHandler] <= $writeFile: {}", path.display());

		let bytes = base64::decode(content_b64)
			.map_err(|e| rpc_error(format!("Invalid base64 content: {}", e), Some("EBADMSG")))?;

		let fs_writer:Arc<dyn FsWriter + Send + Sync> = self.runtime.get_environment().require();

		// Pass create/overwrite flags
		fs_writer
			.write_file(&path, bytes, create, overwrite)
			.await
			.map(|_| Value::Null)
			.map_err(|e| map_common_error_to_rpc_string(e, "workspacefs_$writeFile"))
	}

	// TODO: Implement $createDirectory, $delete, $rename, $copy similarly
	// They will parse their specific arguments (path, options like recursive,

	// overwrite) and call the corresponding method on FsWriter.
}

// --- Setup Function ---
pub fn setup_mountain_rpc_server<R:TauriRuntime>(_app_handle:AppHandle<R>, _runtime:Arc<AppRuntime>) {
	info!("[RPC Setup] Mountain RPC handlers are conceptually available for Track dispatcher.");

	// No explicit registration or server start needed here.
	// Track.rs dynamically instantiates these handlers or calls their methods
	// based on RPC method names.
}
