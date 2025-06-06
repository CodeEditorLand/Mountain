
// Contains the primary logic for handling command execution, registration, and
// unregistration.

#![allow(non_snake_case, non_camel_case_types)]

use std::{
	future::Future,
	pin::Pin,
	sync::{Arc, MutexGuard as StdMutexGuard},
};

use Common::{Errors::CommonError, IpcEffect::ProxyConfiguration as ProxyTarget};
use log::{debug, error, info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime as TauriRuntime, State, Window};

use crate::{
	AppState::{AppState, CommandHandler},
	Handlers::ErrorUtils,
	Runtime::AppRuntime,
	Vine,
}; // The gRPC communication module

// A prefix used to identify commands that are delegated to the Cocoon sidecar
// for execution.
const COCOON_DELEGATING_COMMAND_IDENTIFIER_PREFIX:&str = "_cocoon.executeContributedCommandWithCachedArgument";

/// Formats a Mutex lock error into a standardized RPC error string.
fn FormatAppStateLockErrorForRpc<T>(PoisonError:std::sync::PoisonError<StdMutexGuard<'_, T>>, Context:&str) -> String {
	let CommonErrorInstance = CommonError::StateLock {
		Context:format!("[CommandsHandler LockError] Failed lock on {}: {}", Context, PoisonError),
	};
	error!("{}", CommonErrorInstance);
	ErrorUtils::MapCommonErrorToRpcString(CommonErrorInstance, Context)
}

/// Handles the registration of a command proxied from a sidecar.
pub async fn HandleRegisterCommand<R:TauriRuntime>(
	ApplicationHandle:AppHandle<R>,
	SidecarIdentifier:String,
	Parameters:Value,
) -> Result<Value, String> {
	let CommandIdentifier = Parameters
		.get("id")
		.and_then(Value::as_str)
		.ok_or_else(|| ErrorUtils::RpcParamErrorString("HandleRegisterCommand", "id", "string", None))?
		.to_string();

	info!(
		"[CommandHandler] Registering PROXY command '{}' from sidecar '{}'",
		CommandIdentifier, SidecarIdentifier
	);
	let AppStateInstance = ApplicationHandle.state::<AppState>();
	let mut Registry = AppStateInstance
		.CommandRegistry
		.lock()
		.map_err(|Error| FormatAppStateLockErrorForRpc(Error, "CommandRegistry for register"))?;

	if Registry.contains_key(&CommandIdentifier) {
		warn!(
			"[CommandHandler] Warning: Command identifier '{}' is already registered. Overwriting.",
			CommandIdentifier
		);
	}

	Registry.insert(
		CommandIdentifier.clone(),
		CommandHandler::Proxied {
			SidecarIdentifier:SidecarIdentifier.clone(),
			CommandIdentifier:CommandIdentifier.clone(),
		},
	);
	info!(
		"[CommandHandler] Command '{}' (proxy for '{}') registered.",
		CommandIdentifier, SidecarIdentifier
	);
	Ok(Value::Null)
}

/// Handles the unregistration of a command previously registered by a sidecar.
pub async fn HandleUnregisterCommand<R:TauriRuntime>(
	ApplicationHandle:AppHandle<R>,
	SidecarIdentifier:String,
	Parameters:Value,
) -> Result<Value, String> {
	let CommandIdentifierString = Parameters
		.get("id")
		.and_then(Value::as_str)
		.ok_or_else(|| ErrorUtils::RpcParamErrorString("HandleUnregisterCommand", "id", "string", None))?;

	info!(
		"[CommandHandler] Unregistering command '{}' requested by sidecar '{}'",
		CommandIdentifierString, SidecarIdentifier
	);
	let AppStateInstance = ApplicationHandle.state::<AppState>();
	let mut Registry = AppStateInstance
		.CommandRegistry
		.lock()
		.map_err(|Error| FormatAppStateLockErrorForRpc(Error, "CommandRegistry for unregister"))?;

	if Registry.remove(CommandIdentifierString).is_some() {
		info!("[CommandHandler] Command '{}' unregistered.", CommandIdentifierString);
	} else {
		warn!(
			"[CommandHandler] Command '{}' not found for unregistration (requested by {}).",
			CommandIdentifierString, SidecarIdentifier
		);
	}
	Ok(Value::Null)
}

/// Retrieves a list of all registered command identifiers.
pub async fn HandleGetCommands<R:TauriRuntime>(
	ApplicationHandle:AppHandle<R>,
	_RuntimeState:State<'_, Arc<AppRuntime>>, // Kept for signature consistency, might be used later
) -> Result<Value, String> {
	debug!("[CommandHandler] Handling GetCommands request");
	let AppStateInstance = ApplicationHandle.state::<AppState>();
	let Registry = AppStateInstance
		.CommandRegistry
		.lock()
		.map_err(|Error| FormatAppStateLockErrorForRpc(Error, "CommandRegistry for GetCommands"))?;
	let CommandList:Vec<String> = Registry.keys().cloned().collect();
	Ok(json!(CommandList))
}

/// Executes a command by its identifier, dispatching to either a native or
/// proxied handler.
pub async fn HandleExecuteCommand<R:TauriRuntime>(
	ApplicationHandle:AppHandle<R>,
	Window:Window<R>,
	Runtime:State<'_, Arc<AppRuntime>>,
	Parameters:Value,
) -> Result<Value, String> {
	let CommandIdentifierToExecute = Parameters
		.get("id")
		.and_then(Value::as_str)
		.ok_or_else(|| ErrorUtils::RpcParamErrorString("HandleExecuteCommand", "params.id", "string", None))?
		.to_string();

	let OriginalArgumentValue = Parameters.get("args").cloned().unwrap_or(Value::Null);

	info!(
		"[CommandHandler] Execute: Identifier='{}', ArgumentType='{:?}'",
		CommandIdentifierToExecute,
		OriginalArgumentValue.kind()
	);
	trace!(
		"[CommandHandler] Full arguments for {}: {:?}",
		CommandIdentifierToExecute, OriginalArgumentValue
	);

	if CommandIdentifierToExecute.starts_with(COCOON_DELEGATING_COMMAND_IDENTIFIER_PREFIX) {
		let IdentifierArgumentArray = OriginalArgumentValue.as_array().ok_or_else(|| {
			ErrorUtils::RpcErrorString(
				format!(
					"Delegating command '{}' expects arguments to be an array.",
					CommandIdentifierToExecute
				),
				Some("EBADARG_DELEGATE_CMD"),
			)
		})?;
		let IdentifierString = IdentifierArgumentArray.get(0).and_then(Value::as_str).ok_or_else(|| {
			ErrorUtils::RpcErrorString(
				format!(
					"Delegating command '{}' received invalid identifier (not a string or missing).",
					CommandIdentifierToExecute
				),
				Some("EBADARG_DELEGATE_IDENT"),
			)
		})?;

		info!(
			"[CommandHandler] Detected delegating command '{}'. Routing to Cocoon.",
			CommandIdentifierToExecute
		);
		let RpcParametersForCocoon = json!([IdentifierString, []]);
		let RpcMethodOnCocoon = format!("{}$executeContributedCommand", ProxyTarget::ExtHostCommands.GetTargetPrefix());

		return Vine::SendRequest("cocoon-main".to_string(), RpcMethodOnCocoon, RpcParametersForCocoon, 30000)
			.await
			.map_err(|Error| {
				ErrorUtils::RpcErrorString(
					format!("Failed to execute delegated command on Cocoon: {}", Error),
					Some("EIPC_DELEGATE_EXEC_FAIL"),
				)
			});
	}

	let AppStateInstance = ApplicationHandle.state::<AppState>();
	let HandlerInformationOption = {
		let RegistryGuard = AppStateInstance
			.CommandRegistry
			.lock()
			.map_err(|Error| FormatAppStateLockErrorForRpc(Error, "CommandRegistry for execute"))?;
		RegistryGuard.get(&CommandIdentifierToExecute).cloned()
	};

	match HandlerInformationOption {
		Some(CommandHandler::Native(NativeHandlerFunction)) => {
			debug!("[CommandHandler] Executing NATIVE command '{}'.", CommandIdentifierToExecute);
			NativeHandlerFunction(ApplicationHandle, Window, Runtime.inner().clone(), OriginalArgumentValue).await
		},
		Some(CommandHandler::Proxied { SidecarIdentifier, CommandIdentifier: ProxiedCommandIdentifier }) => {
			debug!(
				"[CommandHandler] Executing PROXIED command '{}' (as '{}') on sidecar '{}'.",
				CommandIdentifierToExecute, ProxiedCommandIdentifier, SidecarIdentifier
			);
			let RpcParametersForCocoon = json!([ProxiedCommandIdentifier, OriginalArgumentValue]);
			let RpcMethodOnCocoon =
				format!("{}$executeContributedCommand", ProxyTarget::ExtHostCommands.GetTargetPrefix());

			Vine::SendRequest(&SidecarIdentifier, RpcMethodOnCocoon, RpcParametersForCocoon, 30000)
				.await
				.map_err(|Error| {
					ErrorUtils::RpcErrorString(
						format!(
							"Failed to execute proxied command '{}' on sidecar '{}': {}",
							CommandIdentifierToExecute, SidecarIdentifier, Error
						),
						Some("EIPC_PROXY_EXEC_FAIL"),
					)
				})
		},
		None => {
			error!(
				"[CommandHandler] Command '{}' not found in registry.",
				CommandIdentifierToExecute
			);
			Err(ErrorUtils::RpcErrorString(
				format!("Command '{}' not found.", CommandIdentifierToExecute),
				Some("ENOCMD_EXEC"),
			))
		},
	}
}

/// A native command handler for saving all documents.
pub fn HandleNativeSaveAll<R:TauriRuntime>(
	_ApplicationHandle:AppHandle<R>,
	_Window:Window<R>,
	Runtime:Arc<AppRuntime>,
	Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		let IncludeUntitled = Argument.get(0).and_then(Value::as_bool).unwrap_or(true);
		info!(
			"[NativeCommand] Executing 'workbench.action.files.saveAll' (IncludeUntitled: {})",
			IncludeUntitled
		);
		let Effect = Common::WorkspaceEffect::SaveAllDocuments(IncludeUntitled);
		Runtime
			.Run(Effect)
			.await
			.map(|ResultsVectorBool| json!(ResultsVectorBool))
			.map_err(|Error| ErrorUtils::MapCommonErrorToRpcString(Error, "native_command_save_all"))
	})
}

/// A native command handler for showing the "About" dialog.
pub fn HandleNativeShowAbout<R:TauriRuntime>(
	ApplicationHandle:AppHandle<R>,
	_Window:Window<R>,
	Runtime:Arc<AppRuntime>,
	_Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		info!("[NativeCommand] Executing 'mountain.action.showAbout'");
		let Version = ApplicationHandle.package_info().version.to_string();
		let AppName = &ApplicationHandle.package_info().name;
		let Message = format!("{} (Mountain)\nVersion: {}\n\nMore info at our website.", AppName, Version);
		let Effect = Common::UiEffect::ShowMessage(Common::UiEffect::MessageSeverity::Info, Message, Value::Null);
		Runtime
			.Run(Effect)
			.await
			.map(|OptionalStringResult| json!(OptionalStringResult))
			.map_err(|Error| ErrorUtils::MapCommonErrorToRpcString(Error, "native_command_show_about"))
	})
}

/// Registers a native command handler into the command registry.
pub fn RegisterNativeCommandInternal<R:TauriRuntime + 'static>(
	Registry:&mut HashMap<String, CommandHandler<R>>,
	CommandIdentifier:String,
	Handler:fn(
		AppHandle<R>,
		Window<R>,
		Arc<AppRuntime>,
		Value,
	) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>,
) {
	if Registry.contains_key(&CommandIdentifier) {
		warn!(
			"[CommandRegistry] Native command '{}' is already registered. Overwriting.",
			CommandIdentifier
		);
	}
	info!("[CommandRegistry] Registered native command: {}", CommandIdentifier);
	Registry.insert(CommandIdentifier, CommandHandler::Native(Handler));
}
