use std::sync::Arc;

use log::{debug, error};
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime, State, command};

/// @module TrackLogic
/// @description Contains the main dispatch functions for routing all incoming
/// commands and RPC requests to the appropriate execution logic.
use super::EffectCreation;
use crate::runtime::AppRuntime::AppRuntime;
use crate::{handlers, rpc}; // Assuming a future rpc module for fallback

/// The primary Tauri command handler for requests originating from the Sky
/// frontend.
///
/// This function receives a command name and arguments, attempts to create a
/// corresponding `ActionEffect`, and then uses the `AppRuntime` to execute it.
///
/// @param Command - The name of the command to dispatch.
/// @param Argument - A `serde_json::Value` containing the command's arguments.
/// @returns A `Result` containing the successful JSON value or an error string.
#[command]
pub async fn DispatchCommand<R:Runtime>(
	AppHandle:AppHandle<R>,
	Runtime:State<'_, Arc<AppRuntime>>,
	Command:String,
	Argument:Value,
) -> Result<Value, String> {
	debug!("[Track] Dispatching frontend command: {}", Command);
	match EffectCreation::CreateEffectForFrontendCommand(&AppHandle, &Command, Argument) {
		Ok(Effect) => Runtime.Run(Effect).await.map_err(|e| e.to_string()),
		Err(e) => Err(e),
	}
}

/// The primary dispatcher for requests originating from a Cocoon sidecar via
/// gRPC.
///
/// This function implements a two-tiered dispatch strategy:
/// 1. It first attempts to map the RPC `MethodName` to a declarative
///    `ActionEffect`. This is the preferred execution path.
/// 2. If no matching `ActionEffect` can be created, it falls back to a direct
///    RPC handler system for legacy or special cases.
///
/// @param SidecarIdentifier - The ID of the sidecar that sent the request.
/// @param MethodName - The RPC method to invoke.
/// @param Parameters - A `serde_json::Value` containing the method's
/// parameters. @returns A `Result` containing the successful JSON value or an
/// error string.
pub async fn DispatchSidecarRequest<R:Runtime>(
	AppHandle:AppHandle<R>,
	Runtime:Arc<AppRuntime>,
	SidecarIdentifier:String,
	MethodName:String,
	Parameters:Value,
) -> Result<Value, String> {
	debug!(
		"[Track] Dispatching sidecar request from '{}': {}",
		SidecarIdentifier, MethodName
	);

	// Attempt to create an ActionEffect first.
	match EffectCreation::CreateEffectForSidecarRequest(&MethodName, &Parameters) {
		Ok(Effect) => Runtime.Run(Effect).await.map_err(|e| e.to_string()),
		Err(_) => {
			// Fallback to direct RPC handler logic if no effect is mapped.
			debug!("[Track] No effect found for '{}', falling back to RPC handler.", MethodName);
			// This `rpc::RouteRpcCall` would be the entry to a system that directly
			// calls methods on handler structs, bypassing the Effect system.
			rpc::RouteRpcCall(AppHandle, Runtime, SidecarIdentifier, MethodName, Parameters).await
		},
	}
}
