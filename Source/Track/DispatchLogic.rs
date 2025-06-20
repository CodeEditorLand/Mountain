//! # DispatchLogic
//!
//! Contains the main dispatch functions for routing all incoming commands and
//! RPC requests to the appropriate execution logic via the effect system.

use std::sync::Arc;

use log::{debug, error};
use serde_json::Value;
use tauri::{AppHandle, Runtime, State, command};

use super::EffectCreation;
use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

/// The primary Tauri command handler for requests originating from the `Sky`
/// frontend.
#[command]
pub async fn DispatchFrontendCommand<R:Runtime>(
	ApplicationHandle:AppHandle<R>,
	RunTime:State<'_, Arc<ApplicationRunTime>>,
	Command:String,
	Argument:Value,
) -> Result<Value, String> {
	debug!("[DispatchLogic] Dispatching frontend command: {}", Command);
	match EffectCreation::CreateEffectForRequest(&ApplicationHandle, &Command, Argument) {
		Ok(EffectFn) => {
			let runtime_clone = RunTime.inner().clone();
			EffectFn(runtime_clone).await
		},
		Err(e) => {
			error!("[DispatchLogic] Failed to create effect for command '{}': {}", Command, e);
			Err(e)
		},
	}
}

/// The primary dispatcher for requests originating from a `Cocoon` sidecar via
/// gRPC.
pub async fn DispatchSidecarRequest<R:Runtime>(
	ApplicationHandle:AppHandle<R>,
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
		Ok(EffectFn) => EffectFn(RunTime).await,
		Err(e) => {
			error!(
				"[DispatchLogic] Failed to create effect for sidecar method '{}': {}",
				MethodName, e
			);
			Err(e)
		},
	}
}
