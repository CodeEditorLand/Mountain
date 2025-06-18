// @module TrackLogic
// @description Contains the main dispatch functions for routing all incoming
// commands and RPC requests to the appropriate execution logic.

use std::sync::Arc;

use log::{debug, error};
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime, State, command};

use super::EffectCreation;
use crate::{Handler, RunTime::ApplicationRunTime::ApplicationRunTime};

// The primary Tauri command handler for requests originating from the Sky
// frontend.
//
// This function receives a command name and arguments, attempts to create a
// corresponding `ActionEffect`, and then uses the `ApplicationRunTime` to
// execute it.
//
// @param Command - The name of the command to dispatch (e.g., "fs.readFile").
// @param Argument - A `serde_json::Value` containing the command's arguments.
// @returns A `Result` containing the successful JSON value or an error string.
#[command]
pub async fn DispatchCommand<R:Runtime>(
	app_handle:AppHandle<R>,
	run_time:State<'_, Arc<ApplicationRunTime>>,
	command:String,
	argument:Value,
) -> Result<Value, String> {
	debug!("[Track] Dispatching frontend command: {}", command);
	match EffectCreation::CreateEffectForFrontendCommand(&app_handle, &command, argument) {
		Ok(effect) => run_time.Run(effect).await.map_err(|e| e.to_string()),
		Err(e) => {
			error!("[Track] Failed to create effect for command '{}': {}", command, e);
			Err(e)
		},
	}
}

// The primary dispatcher for requests originating from a Cocoon sidecar via
// gRPC.
//
// This function implements a two-tiered dispatch strategy:
// 1. It first attempts to map the RPC `MethodName` to a declarative
//    `ActionEffect`. This is the preferred execution path.
// 2. If no matching `ActionEffect` can be created, it falls back to a direct
//    RPC handler system for legacy or special cases.
//
// @param sidecar_identifier - The ID of the sidecar that sent the request.
// @param method_name - The RPC method to invoke.
// @param parameters - A `serde_json::Value` containing the method's
// parameters.
// @returns A `Result` containing the successful JSON value or an error string.
pub async fn DispatchSidecarRequest<R:Runtime>(
	app_handle:AppHandle<R>,
	run_time:Arc<ApplicationRunTime>,
	sidecar_identifier:String,
	method_name:String,
	parameters:Value,
) -> Result<Value, String> {
	debug!(
		"[Track] Dispatching sidecar request from '{}': {}",
		sidecar_identifier, method_name
	);

	// Attempt to create an ActionEffect first.
	match EffectCreation::CreateEffectForSidecarRequest(&method_name, &parameters) {
		Ok(effect) => run_time.Run(effect).await.map_err(|e| e.to_string()),
		Err(_) => {
			// Fallback to direct RPC handler logic if no effect is mapped.
			debug!(
				"[Track] No effect found for '{}', falling back to direct RPC handler.",
				method_name
			);
			// This is where you would call a different system for non-effect-based RPCs.
			// For this project, we assume most things can be effects, but this provides an
			// escape hatch.
			Handler::rpc::RouteRPCCall(app_handle, run_time, sidecar_identifier, method_name, parameters).await
		},
	}
}
