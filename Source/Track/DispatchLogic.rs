//! # DispatchLogic
//!
//! Contains the main dispatch functions for routing all incoming commands and
//! RPC requests to the appropriate execution logic via the effect system.

use std::sync::Arc;

use log::{debug, error};
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime, State, command};

use super::EffectCreation;
use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

/// The primary Tauri command handler for requests originating from the `Sky`
/// frontend.
///
/// This function receives a command name and arguments, attempts to create a
/// corresponding `ActionEffect`, and then uses the `ApplicationRunTime` to
/// execute it.
///
/// # Parameters
/// * `Command`: The name of the command to dispatch (e.g.,
///   "FileSystem.ReadFile").
/// * `Argument`: A `serde_json::Value` containing the command's arguments.
///
/// # Returns
/// A `Result` containing the successful JSON value or an error string.
#[command]
pub async fn DispatchFrontendCommand(
	ApplicationHandle:AppHandle,
	RunTime:State<'_, Arc<ApplicationRunTime>>,
	Command:String,
	Argument:Value,
) -> Result<Value, String> {
	debug!("[DispatchLogic] Dispatching frontend command: {}", Command);
	match EffectCreation::CreateEffectForRequest(&ApplicationHandle, &Command, Argument) {
		Ok(Effect) => RunTime.Run(Effect).await.map_err(|e| format!("Effect execution failed: {}", e)),
		Err(e) => {
			error!("[DispatchLogic] Failed to create effect for command '{}': {}", Command, e);
			Err(e)
		},
	}
}

/// The primary dispatcher for requests originating from a `Cocoon` sidecar via
/// gRPC.
///
/// This function maps the RPC `MethodName` to a declarative `ActionEffect` and
/// executes it.
///
/// # Parameters
/// * `SidecarIdentifier`: The ID of the sidecar that sent the request.
/// * `MethodName`: The RPC method to invoke.
/// * `Parameters`: A `serde_json::Value` containing the method's parameters.
///
/// # Returns
/// A `Result` containing the successful JSON value or an error string.
pub async fn DispatchSidecarRequest(
	ApplicationHandle:AppHandle,
	RunTime:Arc<ApplicationRunTime>,
	SidecarIdentifier:String,
	MethodName:String,
	Parameters:Value,
) -> Result<Value, String> {
	debug!(
		"[DispatchLogic] Dispatching sidecar request from '{}': {}",
		SidecarIdentifier, MethodName
	);

	match EffectCreation::CreateEffectForRequest(&ApplicationHandle, &MethodName, Parameters) {
		Ok(Effect) => RunTime.Run(Effect).await.map_err(|e| format!("Effect execution failed: {}", e)),
		Err(e) => {
			error!(
				"[DispatchLogic] Failed to create effect for sidecar method '{}': {}",
				MethodName, e
			);
			Err(e)
		},
	}
}
