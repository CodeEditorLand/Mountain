// @module rpc (Handler)
// @description Provides a fallback mechanism for handling RPC calls that are
// not mapped to the `ActionEffect` system. This allows for direct invocation of
// handler logic for special or legacy cases.

#![allow(non_snake_case)]

use std::sync::Arc;

use log::warn;
use serde_json::Value;
use tauri::{AppHandle, Runtime};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

// The main entry point for routing non-effect-based RPC calls.
pub async fn RouteRPCCall<R:Runtime>(
	app_handle:AppHandle<R>,
	_runtime:Arc<ApplicationRunTime>,
	_sidecar_identifier:String,
	method_name:String,
	parameters:Value,
) -> Result<Value, String> {
	warn!("[RPC Fallback] Routing method '{}' via direct handler.", method_name);

	// This is where you would have a large match statement to call specific
	// handler functions directly. This provides an escape hatch from the effect
	// system.
	match method_name.as_str() {
		// Example: A special method to handle extension status notifications directly.
		"$onDidActivateExtension" | "$onExtensionActivationError" => {
			crate::Handler::extension_status::HandleExtensionHostStatusNotification(
				&app_handle,
				&method_name,
				parameters,
			)
			.await
		},

		// Example: Terminal methods often involve complex state and PTY management
		// that might be handled directly instead of through effects.
		"$createTerminal" => {
			let result = crate::Handler::terminal::CreateTerminalLogic(&app_handle, parameters).await;
			result.map_err(|e| e.to_string())
		},
		"$sendTextToTerminal" => {
			let id = parameters.get(0).and_then(Value::as_u64).ok_or("Invalid terminal ID")?;
			let text = parameters.get(1).and_then(Value::as_str).ok_or("Invalid text")?.to_string();
			let result = crate::Handler::terminal::SendTextToTerminalLogic(&app_handle, id, text).await;
			result.map(|_| Value::Null).map_err(|e| e.to_string())
		},
		"$disposeTerminal" => {
			let id = parameters.get(0).and_then(Value::as_u64).ok_or("Invalid terminal ID")?;
			let result = crate::Handler::terminal::DisposeTerminalLogic(&app_handle, id).await;
			result.map(|_| Value::Null).map_err(|e| e.to_string())
		},

		// Default case for unhandled methods.
		_ => {
			let err_msg = format!("No direct RPC handler found for method: {}", method_name);
			log::error!("{}", err_msg);
			Err(err_msg)
		},
	}
}
