// ---------------------------------------------------------------------------------------------
// Mountain RPC Handlers 
// --------------------------------------------------------------------------------------------
// Defines the server-side RPC method implementations that Mountain exposes *to*
// the Cocoon sidecar. These handlers mirror the `MainThread...Shape` interfaces
// defined in VS Code's `src/vs/platform/extensions/common/extHost.protocol.ts`.
//
// **ROLE AND EVOLUTION:**
// Initially, this module might have contained direct implementations for many
// RPC methods. However, as the architecture evolved towards using an effect
// system (`ActionEffect` processed by `AppRuntime`), many common operations
// (like configuration, storage, secrets, language feature registration,

// document operations, diagnostics) are now preferably handled by creating
// specific effects directly in `track.rs`.
//
// Consequently, many handler methods within the `MainThread...Handler` structs
// in this file might now be:
// 1. **Stubs:** Logging a warning that they were called and indicating that an
//    effect in `track.rs` should have handled the request. This acts as a
//    safeguard if `track.rs` fails to map a method to an effect.
// 2. **Direct Implementations:** For RPCs that don't fit the effect model well,

//    are complex, or involve direct calls to `handlers::*` functions. Examples
//    include `MainThreadCommandsHandler` (which calls `handlers::commands`),

//    `MainThreadLogHandler`, and `MainThreadFileSystemApiHandler`.
// 3. **Delegations to Effect:** Some RPC methods here might still create and
//    run effects using `self.runtime.run()` if they represent higher-level UI
//    operations not directly covered by `track.rs`'s effect creation logic for
//    sidecar requests (e.g., `MainThreadDialogsHandler`,

//    `MainThreadMessageHandler`).
//
// The `track.rs` dispatcher attempts to map incoming sidecar requests to
// effects first. If no effect mapping is found, it then falls back to invoking
// methods on the handler structs defined in this `rpc.rs` module.
//
// Responsibilities:
// - Defining handler structs (e.g., `MainThreadCommandsHandler`,

//   `MainThreadWorkspaceHandler`) that group related RPC methods. Each handler
//   struct typically holds an `AppHandle` and an `Arc<AppRuntime>`.
// - Implementing `async fn methodName(&self, args: Value) -> Result<Value,

//   String>` for each relevant RPC method from `extHost.protocol.ts`.
// - Parsing `serde_json::Value` arguments received from Cocoon (usually an
//   array).
// - Calling specific functions in `handlers::*` submodules (e.g.,

//   `handlers::commands::*`, `handlers::workspace::*`,

//   `handlers::terminal::*`).
// - For UI-related RPCs not handled by `track.rs` effects, creating and
//   dispatching `ActionEffect`s (e.g., `ui_effects::show_message`) via the
//   `self.runtime` (an `Arc<AppRuntime>`).
// - Providing the concrete implementation for the `vscode.workspace.fs` API via
//   `MainThreadFileSystemApiHandler`, which delegates to `FsReader`/`FsWriter`
//   traits from the `MountainEnvironment`.
// - Returning `Ok(Value)` for successful operations or a structured JSON-RPC
//   error string (using `handlers::error_utils`) for failures.
//
// Key Interactions:
// - Handler struct methods are invoked by `track::dispatch_sidecar_request` as
//   a fallback if no direct effect mapping is found.
// - Handler structs use `self.app_handle` for Tauri operations and
//   `self.runtime` for executing effects.
// - Frequently calls functions in various `handlers::*` modules.
// - The contract for method names and argument/result DTOs is largely defined
//   by VS Code's `extHost.protocol.ts`.
// - Uses `handlers::error_utils` for consistent error formatting in RPC
//   responses.
// --------------------------------------------------------------------------------------------

use std::{path::PathBuf, sync::Arc};

// Land_Common imports:
// - `CommonError` for mapping by `error_utils`.
// - `FsReader`/`FsWriter` for `MainThreadFileSystemApiHandler`.
// - `ui_effects` (and DTOs like `MessageSeverity`, `OpenDialogOptions`) for dialog/message handlers.
use Land_Common::{
	errors::CommonError,

	fs_effects::{FsReader, FsWriter},

	ui_effects::{self, MessageSeverity, OpenDialogOptions, SaveDialogOptions},
};
use log::{debug, error, info, trace, warn};
// `serde::Deserialize` might be used for specific DTOs if args are complex objects.
// use serde::Deserialize;
use serde_json::{Value, json};
// Tauri essentials
use tauri::{AppHandle, Emitter, Manager, Runtime as TauriRuntime, Wry};

use crate::{
	// `AppState` is accessed indirectly via `AppHandle` or `AppRuntime`.
	// app_state::AppState,

	// Access to specific handler functions (e.g., handlers::commands)
	handlers,

	// Centralized RPC error utilities
	handlers::error_utils,

	runtime::AppRuntime, /* For running effects from some RPC handlers
	                      * `vine` is not directly used by RPC methods themselves; Track/Vine handle IPC. */
};

/// Helper to convert a `PathBuf` to a `UriComponents` JSON `Value` DTO.
///
/// This is used for formatting responses from dialog handlers that return file
/// paths, ensuring consistency with VS Code's DTO expectations.
///
/// # Argument
/// * `path` - The `PathBuf` to convert.
///
/// # Returns
/// A `serde_json::Value` representing the `UriComponents` DTO.
fn file_path_to_uri_components_dto(path:&PathBuf) -> Value {
	let uri_str_result = url::Url::from_file_path(path);

	let (scheme, path_str, external_str, fs_path_str) = match uri_str_result {
		Ok(url) => {
			(
				url.scheme().to_string(),
				url.path().to_string(),
				url.to_string(),
				path.to_string_lossy().into_owned(),
			)
		},

		Err(e_url) => {
			warn!(
				"[RPC Helper PathToUri] Failed to create file URL from path '{}': {}. Using fallback representation.",
				path.display(),
				e_url
			);

			(
				// Assume file scheme
				"file".to_string(),
				path.to_string_lossy().into_owned(),
				format!("file:///{}", path.to_string_lossy().replace('\\', "/")),
				path.to_string_lossy().into_owned(),
			)
		},
	};

	json!({
		// Standard VS Code DTO marker for revival
		"$mid": 1,

		"scheme": scheme,

		// Percent-encoded path component from URL
		"path": path_str,

		// Full URI string
		"external": external_str,

		// OS-specific filesystem path
		"fsPath": fs_path_str
	})
}

// --- MainThread Handler Struct Definitions ---
// Each struct groups RPC methods related to a specific `MainThread...Shape`
// interface. They all hold `AppHandle` and `Arc<AppRuntime>` for context and
// effect execution.

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

	// Though log might not run effects, kept for consistency
	pub runtime:Arc<AppRuntime>,
}

#[derive(Clone)]
pub struct MainThreadExtensionServiceHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

// These handlers are mostly placeholders as `track.rs` calls specific
// `handlers::*` functions directly or creates effects for these domains.
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

// These handlers might still create effects for UI interactions.
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

// For window focus, etc.
#[derive(Clone)]
pub struct MainThreadWindowHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

// For status bar item management
#[derive(Clone)]
pub struct MainThreadStatusBarHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

// For vscode.workspace.fs API
#[derive(Clone)]
pub struct MainThreadFileSystemApiHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

// For integrated terminal management
#[derive(Clone)]
pub struct MainThreadTerminalServiceHandler {
	pub app_handle:AppHandle<Wry>,

	pub runtime:Arc<AppRuntime>,
}

// --- Method Implementations for Handler Structs ---

impl MainThreadCommandsHandler {
	/// Handles `$executeCommand` RPC from Cocoon.
	/// `args`: `[commandId: string, ...commandArgument: any[]]`
	pub async fn executeCommand(&self, args:Value) -> Result<Value, String> {
		debug!(
			"[RPC MainThreadCommands] <= $executeCommand: (args sample) '{}...'",
			args.to_string().chars().take(100).collect::<String>()
		);

		let main_window = self.app_handle.get_webview_window("main").ok_or_else(|| {
			error_utils::rpc_error_string(
				"Main window not found for command execution".to_string(),
				Some("ENOWINDOW_CMDEXEC"),
			)
		})?;

		let args_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$executeCommand", "args", "array", None))?;

		let command_id_str = args_array
			.get(0)
			.and_then(Value::as_str)
			.ok_or_else(|| {
				error_utils::rpc_param_error_string("$executeCommand", "commandId (args[0])", "string", Some(0))
			})?
			.to_string();

		// Remaining elements in `args_array` are the command's parameters.
		let command_params_array_val = args_array.get(1..).map_or_else(Vec::new, |s| s.to_vec());

		// Construct parameters for `handlers::commands::handle_execute_command`
		let execute_handler_params_obj = json!({ "id": command_id_str, "args": command_params_array_val });

		handlers::commands::handle_execute_command(
			self.app_handle.clone(),
			main_window,
			// Pass Arc<AppRuntime> directly
			self.runtime.clone(),
			execute_handler_params_obj,
		)
		.await
	}

	/// Handles `$getCommands` RPC from Cocoon. `args`: `void`
	pub async fn getCommands(&self, _args:Value) -> Result<Value, String> {
		debug!("[RPC MainThreadCommands] <= $getCommands");

		handlers::commands::handle_get_commands(self.app_handle.clone(), self.runtime.clone()).await
	}

	/// Handles `$registerCommand` RPC from Cocoon. `args`: `[id: string]`
	pub async fn registerCommand(&self, args:Value) -> Result<Value, String> {
		let args_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$registerCommand", "args", "array", None))?;

		let command_id_to_register = args_array
			.get(0)
			.and_then(Value::as_str)
			.ok_or_else(|| error_utils::rpc_param_error_string("$registerCommand", "id (args[0])", "string", Some(0)))?
			.to_string();

		info!("[RPC MainThreadCommands] <= $registerCommand: id='{}'", command_id_to_register);

		// `sidecar_id` should ideally be passed by Vine/Track to this RPC method.
		// For now, assuming "cocoon-main" if called via this RPC layer.
		handlers::commands::handle_register_command(
			self.app_handle.clone(),
			// TODO: Make sidecar_id dynamic via Track if possible
			"cocoon-main".to_string(),
			json!({ "id": command_id_to_register }),
		)
		.await
	}

	/// Handles `$unregisterCommand` RPC from Cocoon. `args`: `[id: string]`
	pub async fn unregisterCommand(&self, args:Value) -> Result<Value, String> {
		let args_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$unregisterCommand", "args", "array", None))?;

		let command_id_to_unregister = args_array
			.get(0)
			.and_then(Value::as_str)
			.ok_or_else(|| {
				error_utils::rpc_param_error_string("$unregisterCommand", "id (args[0])", "string", Some(0))
			})?
			.to_string();

		info!(
			"[RPC MainThreadCommands] <= $unregisterCommand: id='{}'",
			command_id_to_unregister
		);

		handlers::commands::handle_unregister_command(
			self.app_handle.clone(),
			// TODO: Make sidecar_id dynamic
			"cocoon-main".to_string(),
			json!({ "id": command_id_to_unregister }),
		)
		.await
	}
}

impl MainThreadWorkspaceHandler {
	/// Handles `$resolveWorkspaceFolder` RPC. `args`: `[uriComponentsDto:
	/// Value]`
	pub async fn resolveWorkspaceFolder(&self, args:Value) -> Result<Value, String> {
		let uri_components_val = args
			.as_array()
			.and_then(|a| a.get(0))
			// Clone the Value for the handler
			.cloned()
			.ok_or_else(|| {


				error_utils::rpc_param_error_string(
					"$resolveWorkspaceFolder",


					"uriComponents DTO (args[0])",


					"Value::Object",


					Some(0),


				)
			})?;

		info!(
			"[RPC MainThreadWorkspace] <= $resolveWorkspaceFolder: uri(external)='{:?}'",
			uri_components_val.get("external")
		);

		// This handler is currently a stub in `handlers::workspace`.
		handlers::workspace::handle_get_workspace_folder_for_uri(self.app_handle.clone(), uri_components_val).await
	}

	/// Handles `$findFiles` RPC. `args`: `[include, exclude?, options?]`
	pub async fn findFiles(&self, args:Value) -> Result<Value, String> {
		debug!(
			"[RPC MainThreadWorkspace] <= $findFiles: (args sample) '{}...'",
			args.to_string().chars().take(100).collect::<String>()
		);

		// `args` is already the array `[include, exclude?, options?]`
		handlers::workspace::handle_find_files(self.app_handle.clone(), args).await
	}

	// Note: `$getWorkspaceFolders` and `$requestWorkspaceTrust` are typically
	// handled by effects created directly in `track.rs`, so they might not need
	// explicit RPC handler methods here if that routing is comprehensive.
}

/// Helper for RPC method stubs that should ideally be handled by effects
/// created directly in `track.rs`.
fn rpc_method_should_be_direct_effect(method_name:&str, args:Value) -> Result<Value, String> {
	warn!(
		"[RPC Handler STUB] Method '{}' was called via RPC fallback. This method should ideally be mapped directly to \
		 an ActionEffect in 'track.rs'. Argument: {:?}",
		method_name, args
	);

	Err(error_utils::rpc_error_string(
		format!(
			"RPC method '{}' should be handled by a direct effect created in Track, not via RPC fallback.",
			method_name
		),
		Some("ENOSYS_EFFECT_EXPECTED"),
	))
}

impl MainThreadConfigurationHandler {
	/// `args`: `[section?: string, overrides?: IConfigurationOverrides,
	///
	///
	/// scopeToLanguage?: boolean]`
	pub async fn getConfiguration(&self, args:Value) -> Result<Value, String> {
		rpc_method_should_be_direct_effect("$getConfiguration", args)
	}

	/// `args`: `[target: ConfigurationTarget, key: string, value: any,
	///
	///
	/// overrides?: IConfigurationOverrides, scopeToLanguage?: boolean]`
	pub async fn updateConfigurationOption(&self, args:Value) -> Result<Value, String> {
		rpc_method_should_be_direct_effect("$updateConfigurationOption", args)
	}

	/// `args`: `[target: ConfigurationTarget, key: string, overrides?:
	/// IConfigurationOverrides, scopeToLanguage?: boolean]`
	pub async fn removeConfigurationOption(&self, args:Value) -> Result<Value, String> {
		rpc_method_should_be_direct_effect("$removeConfigurationOption", args)
	}

	/// `args`: `[key: string, overrides?: IConfigurationOverrides]`
	pub async fn inspect(&self, args:Value) -> Result<Value, String> {
		rpc_method_should_be_direct_effect("$inspect", args)
	}
}

impl MainThreadStorageHandler {
	/// `args`: `[shared: boolean, key: string]` (Note: Cocoon shim sends an
	/// object for these) Cocoon's `extHostStorage.ts` calls `$getValue({scope,
	///
	///
	/// key})` and `$setValue({scope, key}, value)`. `track.rs` converts these
	/// object params into effects.
	pub async fn getValue(&self, args:Value) -> Result<Value, String> {
		rpc_method_should_be_direct_effect("$getValue (Storage)", args)
	}

	pub async fn setValue(&self, args:Value) -> Result<Value, String> {
		rpc_method_should_be_direct_effect("$setValue (Storage)", args)
	}
}

impl MainThreadSecretsHandler {
	/// `args`: `[extensionId: string, key: string]`
	pub async fn getPassword(&self, args:Value) -> Result<Value, String> {
		rpc_method_should_be_direct_effect("$getPassword (Secrets)", args)
	}

	/// `args`: `[extensionId: string, key: string, value: string]`
	pub async fn setPassword(&self, args:Value) -> Result<Value, String> {
		rpc_method_should_be_direct_effect("$setPassword (Secrets)", args)
	}

	/// `args`: `[extensionId: string, key: string]`
	pub async fn deletePassword(&self, args:Value) -> Result<Value, String> {
		rpc_method_should_be_direct_effect("$deletePassword (Secrets)", args)
	}
}

impl MainThreadLogHandler {
	/// Handles `$log` RPC from Cocoon for general logging from extensions.
	/// `args`: `[severity: number (LogLevel enum), args: any[] (message
	/// parts)]`
	pub async fn log(&self, args:Value) -> Result<Value, String> {
		let args_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$log", "args", "array", None))?;

		// VS Code LogLevel enum: Trace=0, Debug=1, Info=2, Warning=3, Error=4,

		// Critical=5, Off=6 Default to Info (2) if parsing fails.
		let level_num = args_array.get(0).and_then(Value::as_u64).unwrap_or(2);

		// Message parts start from index 1.
		let message_parts_val = args_array.get(1).cloned().unwrap_or_else(|| json!([]));

		// Convert message parts (which can be any JSON value) to a single string.
		let message_str = if let Some(parts_arr) = message_parts_val.as_array() {
			parts_arr
				.iter()
				// Convert non-strings via to_string
				.map(|val| val.as_str().unwrap_or_else(|| val.to_string()))
				.collect::<Vec<_>>()
				// Join parts with a space
				.join(" ")
		} else {
			// If `args[1]` is not an array, convert it to string directly.
			message_parts_val
				.as_str()
				.unwrap_or_else(|| message_parts_val.to_string())
				.to_string()
		};

		match level_num {
			// Trace
			0 => trace!("[Cocoon ExtHost Log] {}", message_str),

			// Debug
			1 => debug!("[Cocoon ExtHost Log] {}", message_str),

			// Info
			2 => info!("[Cocoon ExtHost Log] {}", message_str),

			// Warning
			3 => warn!("[Cocoon ExtHost Log] {}", message_str),

			// Error, Critical
			4 | 5 => error!("[Cocoon ExtHost Log] {}", message_str),

			_ => info!("[Cocoon ExtHost Log] (Unknown Level {}) {}", level_num, message_str),
		}
		// Logging is fire-and-forget.
		Ok(Value::Null)
	}
}

impl MainThreadExtensionServiceHandler {
	// Lifecycle notifications (`$onWillActivateExtension`, etc.) are typically
	// handled directly by
	// `handlers::extension_status::handle_ext_host_status_notification`
	// via `track.rs` dispatcher logic for notifications.
	// These are stubs in case `track.rs` routing changes or misses one.
	pub async fn onWillActivateExtension(&self, args:Value) -> Result<Value, String> {
		warn!(
			"[RPC MainThreadExtSvc] $onWillActivateExtension called via RPC fallback (should be a direct \
			 notification). Argument: {:?}",
			args
		);

		// Process with the generic handler if it needs to be called from here too.
		handlers::extension_status::handle_extension_host_status_notification(
			self.app_handle.clone(),
			"$onWillActivateExtension",
			args,
		)
		.await
	}

	// Similar stubs for $onDidActivateExtension, $onExtensionActivationError,

	// $onExtensionRuntimeError
	pub async fn onDidActivateExtension(&self, args:Value) -> Result<Value, String> {
		warn!(
			"[RPC MainThreadExtSvc] $onDidActivateExtension called via RPC fallback. Argument: {:?}",
			args
		);

		handlers::extension_status::handle_extension_host_status_notification(
			self.app_handle.clone(),
			"$onDidActivateExtension",
			args,
		)
		.await
	}

	pub async fn onExtensionActivationError(&self, args:Value) -> Result<Value, String> {
		warn!(
			"[RPC MainThreadExtSvc] $onExtensionActivationError called via RPC fallback. Argument: {:?}",
			args
		);

		handlers::extension_status::handle_extension_host_status_notification(
			self.app_handle.clone(),
			"$onExtensionActivationError",
			args,
		)
		.await
	}

	pub async fn onExtensionRuntimeError(&self, args:Value) -> Result<Value, String> {
		warn!(
			"[RPC MainThreadExtSvc] $onExtensionRuntimeError called via RPC fallback. Argument: {:?}",
			args
		);

		handlers::extension_status::handle_extension_host_status_notification(
			self.app_handle.clone(),
			"$onExtensionRuntimeError",
			args,
		)
		.await
	}
}

// Stubs for other MainThread services (Output, Diagnostics, Documents,

// LanguageFeatures) as their methods are typically handled by effects in
// `track.rs` or direct calls to `handlers::*`.
impl MainThreadOutputServiceHandler {
	// All methods (`$register`, `$append`, `$clear`, `$replace`, `$reveal`,

	// `$close`, `$dispose`) are handled by `handlers::output::handle_*` functions,

	// called by `track.rs`. A catch-all stub could be added if specific fallbacks
	// are needed.
}
impl MainThreadDiagnosticsHandler {
	// `$changeMany`, `$getDiagnostics` are handled by
	// `handlers::diagnostics::handle_*`. `$clear` (for an owner) is an effect
	// created in `track.rs`.
}
impl MainThreadDocumentsHandler {
	// Methods like `$tryOpenDocument`, `$tryCreateDocument`, `$trySaveDocument`,

	// `$trySaveDocumentAs`, `$saveAll` are handled by
	// `handlers::documents::handle_*` functions, called by `track.rs`.
	// Document content changes (`$applyEdits`) are typically effects.
}
impl MainThreadLanguageFeaturesHandler {
	// All `$register...Provider` methods and `$unregister` are handled by effects
	// created in `track.rs`. This is a catch-all if any registration unexpectedly
	// falls through to RPC.
	pub async fn CatchAllRegisterProvider(&self, method_name:&str, args:Value) -> Result<Value, String> {
		rpc_method_should_be_direct_effect(method_name, args)
	}

	/// `args`: `[handle: number]`
	pub async fn unregister(&self, args:Value) -> Result<Value, String> {
		rpc_method_should_be_direct_effect("$unregister (LanguageFeatures)", args)
	}
}

impl MainThreadMessageHandler {
	/// Handles `$showMessage` RPC. `args`: `[severity, message, options?]`
	pub async fn showMessage(&self, args:Value) -> Result<Value, String> {
		let params_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$showMessage", "args", "array", None))?;

		// Severity: 1=Info, 2=Warning, 3=Error (VS Code `Severity` enum values)
		let severity_num = params_array.get(0).and_then(Value::as_u64).ok_or_else(|| {
			error_utils::rpc_param_error_string("$showMessage", "severity (args[0])", "u64 number", Some(0))
		})?;

		let message_str = params_array
			.get(1)
			.and_then(Value::as_str)
			.ok_or_else(|| error_utils::rpc_param_error_string("$showMessage", "message (args[1])", "string", Some(1)))?
			.to_string();

		// `options` can be an object with `modal: boolean` or `items: string[]` (button
		// titles).
		let options_val_opt = params_array.get(2).cloned();

		info!(
			"[RPC MainThreadMessageSvc] <= $showMessage: severity_num={}, message_len={}, options_present={}",
			severity_num,
			message_str.len(),
			options_val_opt.is_some()
		);

		let effect_severity = match severity_num {
			// VS Code Severity.Info
			1 => MessageSeverity::Info,

			// VS Code Severity.Warning
			2 => MessageSeverity::Warning,

			// VS Code Severity.Error
			3 => MessageSeverity::Error,

			s_unknown => {
				warn!(
					"[RPC MainThreadMessageSvc] Unknown severity number {} from $showMessage. Defaulting to Info.",
					s_unknown
				);

				MessageSeverity::Info
			},
		};

		let show_message_effect = ui_effects::show_message(effect_severity, message_str, options_val_opt);

		self.runtime.run(show_message_effect).await.map_err(|common_err| {
			error_utils::map_common_error_to_rpc_string(common_err, "$showMessage effect execution")
		})
	}
}

impl MainThreadDialogsHandler {
	/// `args`: `[options?: OpenDialogOptionsDto]`
	pub async fn showOpenDialog(&self, args:Value) -> Result<Value, String> {
		let params_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$showOpenDialog", "args", "array", None))?;

		// Options object is optional
		let options_dto_val_opt = params_array.get(0).cloned();

		info!(
			"[RPC MainThreadDialogs] <= $showOpenDialog (executing via effect). Options: {:?}",
			options_dto_val_opt
		);

		// Deserialize options from Value into OpenDialogOptions struct.
		let open_dialog_options_parsed:Option<OpenDialogOptions> = options_dto_val_opt
			// If Some(Value), try to parse
			.map(serde_json::from_value)
			// Converts Option<Result<T,E>> to Result<Option<T>,E>
			.transpose()
			.map_err(|e_serde| {


				error_utils::rpc_error_string(
					format!("Invalid OpenDialogOptions DTO for $showOpenDialog: {}", e_serde),


					Some("EBADARG_DIALOG_OPTS"),


				)
			})?;

		let show_open_dialog_effect = ui_effects::show_open_dialog(open_dialog_options_parsed);

		self.runtime
			.run(show_open_dialog_effect)
			.await
			.map(|paths_opt_vec:Option<Vec<PathBuf>>| {
				// Convert Option<Vec<PathBuf>> to JSON: array of UriComponents DTOs, or null.
				json!(
					paths_opt_vec
						.map(|paths_vec| { paths_vec.iter().map(file_path_to_uri_components_dto).collect::<Vec<_>>() })
				)
			})
			.map_err(|common_err| {
				error_utils::map_common_error_to_rpc_string(common_err, "$showOpenDialog effect execution")
			})
	}

	/// `args`: `[options?: SaveDialogOptionsDto]`
	pub async fn showSaveDialog(&self, args:Value) -> Result<Value, String> {
		let params_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$showSaveDialog", "args", "array", None))?;

		let options_dto_val_opt = params_array.get(0).cloned();

		info!(
			"[RPC MainThreadDialogs] <= $showSaveDialog (executing via effect). Options: {:?}",
			options_dto_val_opt
		);

		let save_dialog_options_parsed:Option<SaveDialogOptions> =
			options_dto_val_opt.map(serde_json::from_value).transpose().map_err(|e_serde| {
				error_utils::rpc_error_string(
					format!("Invalid SaveDialogOptions DTO for $showSaveDialog: {}", e_serde),
					Some("EBADARG_DIALOG_OPTS"),
				)
			})?;

		let show_save_dialog_effect = ui_effects::show_save_dialog(save_dialog_options_parsed);

		self.runtime
			.run(show_save_dialog_effect)
			.await
			.map(|path_opt:Option<PathBuf>| {
				// Convert Option<PathBuf> to JSON: UriComponents DTO or null.
				json!(path_opt.map(|p| file_path_to_uri_components_dto(&p)))
			})
			.map_err(|common_err| {
				error_utils::map_common_error_to_rpc_string(common_err, "$showSaveDialog effect execution")
			})
	}
}

impl MainThreadWindowHandler {
	/// `args`: `void` (or `[options?: {preserveFocus?: boolean}]` - current VS
	/// Code seems to send no args)
	pub async fn focusWindow(&self, _args:Value) -> Result<Value, String> {
		info!("[RPC MainThreadWindow] <= $focusWindow");

		if let Some(main_window) = self.app_handle.get_webview_window("main") {
			main_window.set_focus().map_err(|e_tauri| {
				error_utils::rpc_error_string(
					format!("Failed to focus main window: {}", e_tauri),
					Some("EWINDOW_FOCUS"),
				)
			})?;

			Ok(Value::Null)
		} else {
			Err(error_utils::rpc_error_string(
				"Main window not found for $focusWindow".to_string(),
				Some("ENOWINDOW_FOCUS"),
			))
		}
	}
	// TODO: Implement `$openUri` (for opening external URLs) and `$asExternalUri`
	//       (for converting file/other URIs to OS-openable form) if these are
	//       routed as RPCs and not direct effects from `track.rs`.
	//       These would likely use `ui_effects::open_external_url` or Tauri's shell
	// API.
}

impl MainThreadStatusBarHandler {
	/// `args`: `[id: string, alignment: number, priority?: number, text:
	/// string, tooltip?: string, command?: string, color?: string,
	///
	///
	/// backgroundColor?: string, accessibilityInformation?:
	/// IAccessibilityInformation]` The `args` here is the full
	/// `IStatusbarEntryDto` from Cocoon, not an array of individual params.
	/// `track.rs` should pass the DTO directly as `params_val`.
	pub async fn setEntry(&self, status_bar_entry_dto:Value) -> Result<Value, String> {
		let entry_id = status_bar_entry_dto
			.get("id")
			.and_then(Value::as_str)
			.unwrap_or("unknown_statusbar_entry");

		info!("[RPC MainThreadStatusBar] <= $setEntry: id='{}'", entry_id);

		trace!("[RPC MainThreadStatusBar] $setEntry full DTO: {:?}", status_bar_entry_dto);

		// Emit a Tauri event for Sky to update the status bar UI.
		// Sky will need to parse the IStatusbarEntryDto structure.
		if let Err(e_emit) = self.app_handle.emit("mountain://statusbar/set", status_bar_entry_dto) {
			error!(
				"[RPC MainThreadStatusBar] Failed to emit 'mountain://statusbar/set' event for entry '{}': {}",
				entry_id, e_emit
			);

			// Optionally return an error if emission is critical, but $setEntry
			// is often fire-and-forget.
		}
		Ok(Value::Null)
	}

	/// `args`: `[id: string]`
	pub async fn disposeEntry(&self, args:Value) -> Result<Value, String> {
		let params_array = args
			.as_array()
			.ok_or_else(|| error_utils::rpc_param_error_string("$disposeEntry", "args", "array", None))?;

		let entry_id_to_dispose = params_array
			.get(0)
			.and_then(Value::as_str)
			.ok_or_else(|| error_utils::rpc_param_error_string("$disposeEntry", "id (args[0])", "string", Some(0)))?;

		info!("[RPC MainThreadStatusBar] <= $disposeEntry: id='{}'", entry_id_to_dispose);

		if let Err(e_emit) = self
			.app_handle
			.emit("mountain://statusbar/dispose", json!({ "id": entry_id_to_dispose }))
		{
			error!(
				"[RPC MainThreadStatusBar] Failed to emit 'mountain://statusbar/dispose' event for entry '{}': {}",
				entry_id_to_dispose, e_emit
			);
		}
		Ok(Value::Null)
	}
}

impl MainThreadTerminalServiceHandler {
	/// `params_val`: `ICreateTerminalOptions` DTO
	pub async fn createTerminal(&self, params_val:Value) -> Result<Value, String> {
		info!("[RPC MainThreadTerminalSvc] <= $createTerminal");

		// Delegate to the more complete handler in `handlers::terminal`.
		handlers::terminal::handle_create_terminal(self.app_handle.clone(), params_val).await
	}

	/// `params_val`: `[id: number, preserveFocus?: boolean]`
	pub async fn show(&self, params_val:Value) -> Result<Value, String> {
		info!("[RPC MainThreadTerminalSvc] <= $show");

		handlers::terminal::handle_show_terminal(self.app_handle.clone(), params_val).await
	}

	/// `params_val`: `[id: number]`
	pub async fn hide(&self, params_val:Value) -> Result<Value, String> {
		info!("[RPC MainThreadTerminalSvc] <= $hide");

		handlers::terminal::handle_hide_terminal(self.app_handle.clone(), params_val).await
	}

	/// `params_val`: `[id: number, text: string]`
	pub async fn sendText(&self, params_val:Value) -> Result<Value, String> {
		info!("[RPC MainThreadTerminalSvc] <= $sendText");

		handlers::terminal::handle_send_text_to_terminal(self.app_handle.clone(), params_val).await
	}

	/// `params_val`: `[id: number]`
	pub async fn dispose(&self, params_val:Value) -> Result<Value, String> {
		info!("[RPC MainThreadTerminalSvc] <= $dispose");

		handlers::terminal::handle_dispose_terminal(self.app_handle.clone(), params_val).await
	}
	// TODO: Add handlers for other terminal methods if needed, e.g.,

	//       `$resize`, `$getInitialCwd`, `$getShellProcessId`, etc.
	//       These would also delegate to `handlers::terminal` or manage state
	// there.
}

/// Nested module for helpers specific to `MainThreadFileSystemApiHandler`.
mod main_thread_fs_api_helpers {

	// Import from parent `rpc` module
	use super::{PathBuf, Value, error_utils};

	/// Helper to parse `PathBuf` from `UriComponents` DTO for FS API methods.
	/// This is a copy of the one in `workspace_fs_api.rs` for use within this
	/// module if `MainThreadFileSystemApiHandler` methods are called directly.
	pub fn path_from_uri_components_for_fs_api_rpc(
		uri_val:&Value,

		method_name_for_error:&str,
	) -> Result<PathBuf, String> {
		let scheme = uri_val.get("scheme").and_then(Value::as_str).unwrap_or("file");

		match scheme {
			"file" | "" => {
				Ok(PathBuf::from(uri_val.get("path").and_then(Value::as_str).ok_or_else(|| {
					error_utils::rpc_param_error_string(
						method_name_for_error,
						"uriComponents.path",
						"string",
						// Assuming uri_val is the component, not an array index
						None,
					)
				})?))
			},

			_ => {
				Err(error_utils::rpc_error_string(
					format!(
						"FS API method '{}' currently only supports 'file' scheme, got '{}'",
						method_name_for_error, scheme
					),
					Some("ENOTSUP_SCHEME_FSAPI"),
				))
			},
		}
	}
}

impl MainThreadFileSystemApiHandler {
	// These methods are implementations of the `vscode.workspace.fs` provider API.
	// They are called by `track.rs` which maps `workspacefs_*` method names.
	// Parameters are expected as a `Value::Array`.

	/// `params_val`: `[uriComponentsDto: Value]`
	pub async fn stat(&self, params_val:Value) -> Result<Value, String> {
		let uri_components_dto = params_val.get(0).ok_or_else(|| {
			error_utils::rpc_param_error_string("FSAPI $stat", "uriComponents DTO (args[0])", "Value::Object", Some(0))
		})?;

		// Use the helper from the nested module for path parsing.
		let path =
			main_thread_fs_api_helpers::path_from_uri_components_for_fs_api_rpc(uri_components_dto, "FSAPI $stat")?;

		debug!("[RPC MainThreadFsApiHandler] -> stat: {}", path.display());

		let fs_reader_provider:Arc<dyn FsReader + Send + Sync> = self.runtime.get_environment().require();

		fs_reader_provider
			.stat_file(&path)
			.await
			.map(|stat_obj| {
				serde_json::to_value(stat_obj).unwrap_or_else(|e_serde| {
					error!(
						"[RPC MainThreadFsApiHandler stat] Failed to serialize FileSystemStat: {}",
						e_serde
					);

					// Fallback, though should not happen for valid Stat
					Value::Null
				})
			})
			.map_err(|common_err| error_utils::map_common_error_to_rpc_string(common_err, "vscode.workspace.fs.stat"))
	}

	/// `params_val`: `[uriComponentsDto: Value]`
	pub async fn read_directory(&self, params_val:Value) -> Result<Value, String> {
		let uri_components_dto = params_val.get(0).ok_or_else(|| {
			error_utils::rpc_param_error_string(
				"FSAPI $readDirectory",
				"uriComponents DTO (args[0])",
				"Value::Object",
				Some(0),
			)
		})?;

		let path = main_thread_fs_api_helpers::path_from_uri_components_for_fs_api_rpc(
			uri_components_dto,
			"FSAPI $readDirectory",
		)?;

		debug!("[RPC MainThreadFsApiHandler] -> readDirectory: {}", path.display());

		let fs_reader_provider:Arc<dyn FsReader + Send + Sync> = self.runtime.get_environment().require();

		fs_reader_provider
			.read_directory(&path)
			.await
			// Converts Vec<(String, CommonFileType)>
			.map(|entries_vec| json!(entries_vec))
			.map_err(|common_err| {


				error_utils::map_common_error_to_rpc_string(
					common_err,


					"vscode.workspace.fs.readDirectory",


				)
			})
	}

	// Implementations for readFile, writeFile, createDirectory, delete, rename,

	// copy follow the same pattern, calling methods on `FsReader`/`FsWriter` from
	// `self.runtime`. These are largely identical to the
	// `handlers::workspace_fs_api::handle_*` functions, as this RPC struct is an
	// alternative way `track.rs` *could* route to them. For brevity, showing one
	// more and noting the pattern.

	/// `params_val`: `[uriComponentsDto: Value]`
	pub async fn read_file(&self, params_val:Value) -> Result<Value, String> {
		// This is identical to
		// `handlers::workspace_fs_api::handle_workspace_fs_read_file` if called with
		// `self.runtime` and `params_val`.
		handlers::workspace_fs_api::handle_workspace_fs_read_file(self.runtime.clone(), params_val).await
	}

	/// `params_val`: `[uriComponentsDto, contentBase64, optionsDto]`
	pub async fn write_file(&self, params_val:Value) -> Result<Value, String> {
		handlers::workspace_fs_api::handle_workspace_fs_write_file(self.runtime.clone(), params_val).await
	}

	/// `params_val`: `[uriComponentsDto]`
	pub async fn create_directory(&self, params_val:Value) -> Result<Value, String> {
		handlers::workspace_fs_api::handle_workspace_fs_create_directory(self.runtime.clone(), params_val).await
	}

	/// `params_val`: `[uriComponentsDto, optionsDto]`
	pub async fn delete(&self, params_val:Value) -> Result<Value, String> {
		handlers::workspace_fs_api::handle_workspace_fs_delete(self.runtime.clone(), params_val).await
	}

	/// `params_val`: `[sourceUriDto, targetUriDto, optionsDto]`
	pub async fn rename(&self, params_val:Value) -> Result<Value, String> {
		handlers::workspace_fs_api::handle_workspace_fs_rename(self.runtime.clone(), params_val).await
	}

	/// `params_val`: `[sourceUriDto, targetUriDto, optionsDto]`
	pub async fn copy(&self, params_val:Value) -> Result<Value, String> {
		handlers::workspace_fs_api::handle_workspace_fs_copy(self.runtime.clone(), params_val).await
	}
}

// --- Setup Function (Conceptual) ---
/// Conceptually sets up the Mountain RPC server endpoint.
///
/// In practice, `track.rs` acts as the dispatcher and instantiates these
/// handler structs on-demand when an RPC call (that's not an effect) is routed.
/// This function serves to indicate that the RPC handlers defined in this
/// module are available to be used by the dispatching mechanism.
pub fn setup_mountain_rpc_server<R:TauriRuntime>(
	// Parameter kept for consistency, may be used if handlers are pre-instantiated
	_app_handle:AppHandle<R>,

	// Parameter kept for consistency
	_runtime:Arc<AppRuntime>,
) {
	info!("[RPC Setup] Mountain RPC handler structs are available for Track dispatcher.");

	// No explicit server needs to be "started" here because `track.rs` (called
	// by Vine) will instantiate and call methods on these structs directly.
	// If these handlers were to be registered in a generic RPC registry, that
	// would happen here.
}
