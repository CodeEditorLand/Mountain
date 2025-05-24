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
	errors::CommonError,              // Used by error_utils::map_common_error_to_rpc_string
	fs_effects::{FsReader, FsWriter}, // For MainThreadFileSystemApiHandler
	language_feature_effects,         // Potentially for complex/fallback language feature RPCs if any remain
	ui_effects::{self, MessageSeverity, OpenDialogOptions, SaveDialogOptions}, // For Dialogs/Messages
};
use log::{debug, error, info, trace, warn};
use serde::Deserialize;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime as TauriRuntime, State, Window, Wry}; // Window for MainThreadCommands

use crate::{
	app_state::AppState,   // Though not directly used as State<T> here, kept for broader context
	handlers,              // For direct handler calls & error_utils
	handlers::error_utils, // Centralized RPC error utilities
	runtime::AppRuntime,

	vine,
};

// Helper to convert path to UriComponents JSON Value for dialog responses
fn path_to_uri_components_value(path:&PathBuf) -> Value {
	let uri_str = url::Url::from_file_path(path)
		.map(|url| url.to_string())
		.unwrap_or_else(|_| format!("file:///{}", path.to_string_lossy().replace('\\', "/")));

	json!({"scheme": "file", "path": path.to_str().unwrap_or(""), "external": uri_str, "$mid": 1 })
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

		// TODO: Get sidecar_id from request context if becomes multi-sidecar
		let sidecar_id = "cocoon-main".to_string();

		handlers::commands::handle_register_command(self.app_handle.clone(), sidecar_id, json!({ "id": id })).await
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

		let sidecar_id = "cocoon-main".to_string();

		handlers::commands::handle_unregister_command(self.app_handle.clone(), sidecar_id, json!({ "id": id })).await
	}
}

impl MainThreadWorkspaceHandler {
	pub async fn resolveWorkspaceFolder(&self, args:Value) -> Result<Value, String> {
		warn!("[RPC Ws] <= $resolveWorkspaceFolder (delegating to handler): {:?}", args);

		let uri_components_val = args.as_array().and_then(|a| a.get(0)).cloned().ok_or_else(|| {
			error_utils::rpc_param_error_string("$resolveWorkspaceFolder", "uriComponents", "Value", Some(0))
		})?;

		handlers::workspace::handle_get_workspace_folder(self.app_handle.clone(), uri_components_val).await
	}

	pub async fn findFiles(&self, args:Value) -> Result<Value, String> {
		debug!(
			"[RPC Ws] <= $findFiles: '{}...'",
			args.to_string().chars().take(100).collect::<String>()
		);

		handlers::workspace::handle_find_files(self.app_handle.clone(), args).await
	}

	// $getWorkspaceFolders, $requestWorkspaceTrust are typically effects created in
	// track.rs
}

impl MainThreadConfigurationHandler {
	// All configuration methods ($getConfiguration, $updateConfigurationOption,

	// $removeConfigurationOption, $inspect) are effects created directly in
	// track.rs. These stubs are fallbacks in case they are unexpectedly routed
	// here.
	pub async fn getConfiguration(&self, args:Value) -> Result<Value, String> {
		warn!("[RPC Config] $getConfiguration called (should be effect). Args: {:?}", args);

		Err(error_utils::rpc_error_string(
			"$getConfiguration should be an effect created in Track.".to_string(),
			Some("ENOSYS_EFFECT_FALLBACK"),
		))
	}

	pub async fn updateConfigurationOption(&self, args:Value) -> Result<Value, String> {
		warn!(
			"[RPC Config] $updateConfigurationOption called (should be effect). Args: {:?}",
			args
		);

		Err(error_utils::rpc_error_string(
			"$updateConfigurationOption should be an effect created in Track.".to_string(),
			Some("ENOSYS_EFFECT_FALLBACK"),
		))
	}

	pub async fn removeConfigurationOption(&self, args:Value) -> Result<Value, String> {
		warn!(
			"[RPC Config] $removeConfigurationOption called (should be effect). Args: {:?}",
			args
		);

		Err(error_utils::rpc_error_string(
			"$removeConfigurationOption should be an effect created in Track.".to_string(),
			Some("ENOSYS_EFFECT_FALLBACK"),
		))
	}

	pub async fn inspect(&self, args:Value) -> Result<Value, String> {
		warn!("[RPC Config] $inspect called (should be effect). Args: {:?}", args);

		Err(error_utils::rpc_error_string(
			"$inspect should be an effect created in Track.".to_string(),
			Some("ENOSYS_EFFECT_FALLBACK"),
		))
	}
}

impl MainThreadStorageHandler {
	// $getValue and $setValue are effects created in track.rs.
	pub async fn getValue(&self, args:Value) -> Result<Value, String> {
		warn!("[RPC Storage] $getValue called (should be an effect). Args: {:?}", args);

		Err(error_utils::rpc_error_string(
			"$getValue should be an effect created in Track.".to_string(),
			Some("ENOSYS_EFFECT_FALLBACK"),
		))
	}

	pub async fn setValue(&self, args:Value) -> Result<Value, String> {
		warn!("[RPC Storage] $setValue called (should be an effect). Args: {:?}", args);

		Err(error_utils::rpc_error_string(
			"$setValue should be an effect created in Track.".to_string(),
			Some("ENOSYS_EFFECT_FALLBACK"),
		))
	}
}

impl MainThreadSecretsHandler {
	// $getPassword, $setPassword, $deletePassword are effects created in track.rs.
	pub async fn getPassword(&self, args:Value) -> Result<Value, String> {
		warn!("[RPC Secrets] $getPassword called (should be an effect). Args: {:?}", args);

		Err(error_utils::rpc_error_string(
			"$getPassword should be an effect created in Track.".to_string(),
			Some("ENOSYS_EFFECT_FALLBACK"),
		))
	}

	pub async fn setPassword(&self, args:Value) -> Result<Value, String> {
		warn!("[RPC Secrets] $setPassword called (should be an effect). Args: {:?}", args);

		Err(error_utils::rpc_error_string(
			"$setPassword should be an effect created in Track.".to_string(),
			Some("ENOSYS_EFFECT_FALLBACK"),
		))
	}

	pub async fn deletePassword(&self, args:Value) -> Result<Value, String> {
		warn!("[RPC Secrets] $deletePassword called (should be an effect). Args: {:?}", args);

		Err(error_utils::rpc_error_string(
			"$deletePassword should be an effect created in Track.".to_string(),
			Some("ENOSYS_EFFECT_FALLBACK"),
		))
	}
}

impl MainThreadLogHandler {
	pub async fn log(&self, args:Value) -> Result<Value, String> {
		let args_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$log", "args array", "array", None))?;

		let level_num = args_array.get(0).and_then(Value::as_u64).unwrap_or(2); // VS Code LogLevel: Info = 2
		let message_val = args_array.get(1);

		let message_str = match message_val {
			Some(Value::String(s)) => s.clone(),

			Some(Value::Array(arr)) => arr.iter().filter_map(Value::as_str).collect::<Vec<&str>>().join(" "),

			_ => "".to_string(),
		};

		match level_num {
			// VS Code LogLevel mapping
			0 => trace!("[Cocoon EH Log] {}", message_str),     // Trace
			1 => debug!("[Cocoon EH Log] {}", message_str),     // Debug
			2 => info!("[Cocoon EH Log] {}", message_str),      // Info
			3 => warn!("[Cocoon EH Log] {}", message_str),      // Warning
			4 | 5 => error!("[Cocoon EH Log] {}", message_str), // Error, Critical
			_ => info!("[Cocoon EH Log] (Unknown Level {}) {}", level_num, message_str),
		}

		Ok(Value::Null)
	}
}

impl MainThreadExtensionServiceHandler {
	// Notifications like $onWillActivateExtension are handled directly by track.rs
	// routing to handlers::extension_status. These are stubs.
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

impl MainThreadOutputServiceHandler {
	// All methods are routed by track.rs directly to handlers::output.
	// This struct is a placeholder; no methods expected here.
}

impl MainThreadDiagnosticsHandler {
	// Methods like $changeMany are routed by track.rs to handlers::diagnostics.
	// $clear is an effect created in track.rs.
	// This struct is a placeholder; no methods expected here unless for complex
	// fallbacks.
}

impl MainThreadDocumentsHandler {
	// Methods like $tryOpenDocument, $trySaveDocument are routed by track.rs
	// directly to handlers::documents.
	// This struct is a placeholder.
}

// Helper for language feature registration stubs
fn warn_and_error_lang_feat_registration_rpc(method_name:&str, args:Value) -> Result<Value, String> {
	warn!(
		"[RPC LangFeat] {} called (should be an effect created by Track). Args: {:?}",
		method_name, args
	);

	Err(error_utils::rpc_error_string(
		format!("{} should be an effect created in Track.", method_name),
		Some("ENOSYS_EFFECT_FALLBACK"),
	))
}

impl MainThreadLanguageFeaturesHandler {
	// All common $register...Provider methods are now effects created in track.rs.
	// These methods are stubs indicating they should not be reached if track.rs is
	// correct. The method names here match extHost.protocol.ts.
	pub async fn registerHoverProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerHoverProvider", args)
	}

	pub async fn registerCompletionsProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerCompletionsProvider", args)
	}

	pub async fn registerDefinitionProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerDefinitionProvider", args)
	}

	pub async fn registerDeclarationProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerDeclarationProvider", args)
	}

	pub async fn registerImplementationProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerImplementationProvider", args)
	}

	pub async fn registerTypeDefinitionProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerTypeDefinitionProvider", args)
	}

	pub async fn registerCodeActionProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerCodeActionProvider", args)
	}

	pub async fn registerCodeLensProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerCodeLensProvider", args)
	}

	pub async fn registerDocumentFormattingEditProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerDocumentFormattingEditProvider", args)
	}

	pub async fn registerDocumentRangeFormattingEditProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerDocumentRangeFormattingEditProvider", args)
	}

	pub async fn registerOnTypeFormattingEditProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerOnTypeFormattingEditProvider", args)
	}

	pub async fn registerRenameProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerRenameProvider", args)
	}

	pub async fn registerDocumentLinkProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerDocumentLinkProvider", args)
	}

	pub async fn registerDocumentColorProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerDocumentColorProvider", args)
	}

	pub async fn registerFoldingRangeProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerFoldingRangeProvider", args)
	}

	pub async fn registerSelectionRangeProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerSelectionRangeProvider", args)
	}

	pub async fn registerCallHierarchyProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerCallHierarchyProvider", args)
	}

	pub async fn registerTypeHierarchyProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerTypeHierarchyProvider", args)
	}

	pub async fn registerLinkedEditingRangeProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerLinkedEditingRangeProvider", args)
	}

	pub async fn registerInlayHintsProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerInlayHintsProvider", args)
	}

	pub async fn registerDocumentSymbolProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerDocumentSymbolProvider", args)
	}

	pub async fn registerWorkspaceSymbolProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerWorkspaceSymbolProvider", args)
	}

	pub async fn registerReferencesProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerReferencesProvider", args)
	}

	pub async fn registerDocumentHighlightProvider(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerDocumentHighlightProvider", args)
	}

	// Methods with "Support" in their name were often older or internal ways of
	// phrasing, the protocol uses "Provider". Included stubs for common ones found
	// in snippets for completeness.
	pub async fn registerDefinitionSupport(&self, args:Value) -> Result<Value, String> {
		warn_and_error_lang_feat_registration_rpc("$registerDefinitionSupport", args)
	}

	pub async fn unregister(&self, args:Value) -> Result<Value, String> {
		warn!("[RPC LangFeat] $unregister called (should be an effect). Args: {:?}", args);

		Err(error_utils::rpc_error_string(
			"$unregister for language features should be an effect created in Track.".to_string(),
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

		let options_val = params_array.get(2).cloned(); // DTO: MainThreadMessageOptions

		info!(
			"[RPC MsgSvc] <= $showMessage: severity={}, msg_len={}",
			severity_num,
			message.len()
		);

		trace!("[RPC MsgSvc] $showMessage options: {:?}", options_val);

		let severity_effect = match severity_num {
			1 => MessageSeverity::Info,    // VS Code Severity.Info
			2 => MessageSeverity::Warning, // VS Code Severity.Warning
			3 => MessageSeverity::Error,   // VS Code Severity.Error
			s => {
				warn!("[RPC MsgSvc] Unknown severity {} from $showMessage, defaulting to Info.", s);

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

	pub async fn openUri(&self, args:Value) -> Result<Value, String> {
		warn!(
			"[RPC Window] $openUri called (should be an effect or direct handler). Args: {:?}",
			args
		);

		// Example:
		// let uri_dto = args.as_array().and_then(|a| a.get(0)).cloned().ok_or_else(||
		// error_utils::rpc_param_error_string("$openUri", "uri", "Value", Some(0)))?;

		// let options = args.as_array().and_then(|a| a.get(1)).cloned();

		// let effect = ui_effects::open_external(uri_dto, options); // Assuming such an
		// effect exists self.runtime.run(effect).await.map(|success|
		// json!(success)).map_err(|e| error_utils::map_common_error_to_rpc_string(e,

		// "$openUri"))
		Err(error_utils::rpc_error_string(
			"$openUri not fully implemented, should be an effect or direct handler.".to_string(),
			Some("ENOSYS"),
		))
	}

	pub async fn asExternalUri(&self, args:Value) -> Result<Value, String> {
		warn!(
			"[RPC Window] $asExternalUri called (should be an effect or direct handler). Args: {:?}",
			args
		);

		Err(error_utils::rpc_error_string(
			"$asExternalUri not fully implemented, should be an effect or direct handler.".to_string(),
			Some("ENOSYS"),
		))
	}
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

		// Index 4 for 'text' is based on IStatusbarEntry DTO in VS Code's protocol
		let text_val = params_array
			.get(4)
			.cloned()
			.ok_or_else(|| error_utils::rpc_param_error_string("$setEntry", "text", "Value", Some(4)))?;

		info!(
			"[RPC StatusBar] <= $setEntry: id='{}', text(brief)='{}...'",
			id,
			text_val.as_str().unwrap_or("").chars().take(30).collect::<String>()
		);

		trace!("[RPC StatusBar] $setEntry full args: {:?}", args);

		if let Err(e) = self.app_handle.emit_all("mountain://statusbar/set", args.clone()) {
			error!("[RPC StatusBar] Failed to emit statusbar/set event for {}: {}", id, e);

			// Not returning error to client, as this is a fire-and-forget UI
			// update
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

// Nested module for MainThreadFileSystemApiHandler's helper
mod fs_api_helpers {

	use super::{PathBuf, Value, error_utils}; // Use error_utils from parent scope

	pub fn path_from_uri_components_for_fs_api(uri_val:&Value) -> Result<PathBuf, String> {
		let scheme = uri_val.get("scheme").and_then(Value::as_str).unwrap_or("file");

		match scheme {
			"file" | "" => {
				let path_str = uri_val.get("path").and_then(Value::as_str).ok_or_else(|| {
					error_utils::rpc_error_string(
						"Missing 'path' in URI components for FS API".to_string(),
						Some("EBADARG"),
					)
				})?;

				Ok(PathBuf::from(path_str))
			},

			_ => {
				Err(error_utils::rpc_error_string(
					format!("WorkspaceFS API currently only supports 'file' scheme, got '{}'", scheme),
					Some("ENOTSUP"),
				))
			},
		}
	}
}

impl MainThreadFileSystemApiHandler {
	// Method names here are Rust-idiomatic, as called by track.rs mapping from
	// workspacefs_$methodName
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

		debug!("[RPC FsApiHandler] -> readDirectory: {}", path.display());

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

		debug!("[RPC FsApiHandler] -> readFile: {}", path.display());

		let fs_reader:Arc<dyn FsReader + Send + Sync> = self.runtime.get_environment().require();

		fs_reader
			.read_file(&path)
			.await
			.map(|bytes| json!(base64::encode(&bytes))) // Cocoon expects base64 encoded content
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

		let options_val = params_array.get(2).cloned().unwrap_or(Value::Null);

		let create = options_val.get("create").and_then(Value::as_bool).unwrap_or(true);

		let overwrite = options_val.get("overwrite").and_then(Value::as_bool).unwrap_or(false);

		let path = fs_api_helpers::path_from_uri_components_for_fs_api(&uri_components)?;

		debug!("[RPC FsApiHandler] -> writeFile: {}", path.display());

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

		debug!("[RPC FsApiHandler] -> createDirectory: {}", path.display());

		let fs_writer:Arc<dyn FsWriter + Send + Sync> = self.runtime.get_environment().require();

		// VS Code workspace.fs.createDirectory is implicitly recursive.
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

		let options_val = params_array.get(1).cloned().unwrap_or(Value::Null);

		let recursive = options_val.get("recursive").and_then(Value::as_bool).unwrap_or(false);

		let use_trash = options_val.get("useTrash").and_then(Value::as_bool).unwrap_or(false);

		let path = fs_api_helpers::path_from_uri_components_for_fs_api(&uri_components)?;

		debug!(
			"[RPC FsApiHandler] -> delete: {}, recursive: {}, useTrash: {}",
			path.display(),
			recursive,
			use_trash
		);

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

		let options_val = params_array.get(2).cloned().unwrap_or(Value::Null);

		let overwrite = options_val.get("overwrite").and_then(Value::as_bool).unwrap_or(false);

		let source_path = fs_api_helpers::path_from_uri_components_for_fs_api(&source_uri_comp)?;

		let target_path = fs_api_helpers::path_from_uri_components_for_fs_api(&target_uri_comp)?;

		debug!(
			"[RPC FsApiHandler] -> rename: {} to {}, overwrite: {}",
			source_path.display(),
			target_path.display(),
			overwrite
		);

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

		let options_val = params_array.get(2).cloned().unwrap_or(Value::Null);

		let overwrite = options_val.get("overwrite").and_then(Value::as_bool).unwrap_or(false);

		let source_path = fs_api_helpers::path_from_uri_components_for_fs_api(&source_uri_comp)?;

		let target_path = fs_api_helpers::path_from_uri_components_for_fs_api(&target_uri_comp)?;

		debug!(
			"[RPC FsApiHandler] -> copy: {} to {}, overwrite: {}",
			source_path.display(),
			target_path.display(),
			overwrite
		);

		let fs_writer:Arc<dyn FsWriter + Send + Sync> = self.runtime.get_environment().require();

		fs_writer
			.copy(&source_path, &target_path, overwrite)
			.await
			.map(|_| Value::Null)
			.map_err(|e| error_utils::map_common_error_to_rpc_string(e, "fs.copy"))
	}
}

impl MainThreadTerminalServiceHandler {
	// These are stubs; actual terminal implementation would be complex.
	pub async fn createTerminal(&self, params_val:Value) -> Result<Value, String> {
		warn!("[RPC TerminalHandler] $createTerminal STUBBED: {:?}", params_val);

		let name = params_val.get("name").and_then(Value::as_str).unwrap_or("Terminal");

		let terminal_id = rand::random::<u64>(); // Placeholder ID
		Ok(json!({ "id": terminal_id, "name": name }))
	}

	pub async fn show(&self, params_val:Value) -> Result<Value, String> {
		let id = params_val
			.as_array()
			.and_then(|a| a.get(0))
			.and_then(Value::as_u64)
			.ok_or_else(|| error_utils::rpc_param_error_string("$show (terminal)", "terminalId", "u64", Some(0)))?;

		warn!("[RPC TerminalHandler] $show STUBBED for terminal ID: {}", id);

		self.app_handle
			.emit_all(
				"mountain://terminal/reveal",
				json!({

					"id": id,

					"preserveFocus": params_val.as_array().and_then(|a|a.get(1)).and_then(Value::as_bool).unwrap_or(false)
				}),
			)
			.ok();

		Ok(Value::Null)
	}

	pub async fn hide(&self, params_val:Value) -> Result<Value, String> {
		let id = params_val
			.as_array()
			.and_then(|a| a.get(0))
			.and_then(Value::as_u64)
			.ok_or_else(|| error_utils::rpc_param_error_string("$hide (terminal)", "terminalId", "u64", Some(0)))?;

		warn!("[RPC TerminalHandler] $hide STUBBED for terminal ID: {}", id);

		Ok(Value::Null)
	}

	pub async fn sendText(&self, params_val:Value) -> Result<Value, String> {
		let id = params_val
			.as_array()
			.and_then(|a| a.get(0))
			.and_then(Value::as_u64)
			.ok_or_else(|| error_utils::rpc_param_error_string("$sendText (terminal)", "terminalId", "u64", Some(0)))?;

		let text = params_val
			.as_array()
			.and_then(|a| a.get(1))
			.and_then(Value::as_str)
			.ok_or_else(|| error_utils::rpc_param_error_string("$sendText (terminal)", "text", "string", Some(1)))?;

		warn!(
			"[RPC TerminalHandler] $sendText STUBBED for terminal ID: {}, text: '{}...'",
			id,
			text.chars().take(30).collect::<String>()
		);

		Ok(Value::Null)
	}

	pub async fn dispose(&self, params_val:Value) -> Result<Value, String> {
		let id = params_val
			.as_array()
			.and_then(|a| a.get(0))
			.and_then(Value::as_u64)
			.ok_or_else(|| error_utils::rpc_param_error_string("$dispose (terminal)", "terminalId", "u64", Some(0)))?;

		warn!("[RPC TerminalHandler] $dispose STUBBED for terminal ID: {}", id);

		Ok(Value::Null)
	}
}

// --- Setup Function ---
pub fn setup_mountain_rpc_server<R:TauriRuntime>(_app_handle:AppHandle<R>, _runtime:Arc<AppRuntime>) {
	info!("[RPC Setup] Mountain RPC handlers are conceptually available for Track dispatcher.");

	// No explicit registration or server start needed here.
	// Track.rs dynamically instantiates these handlers or calls their methods
	// based on RPC method names, or creates effects directly.
}
