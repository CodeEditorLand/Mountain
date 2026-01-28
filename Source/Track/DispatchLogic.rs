// File: Mountain/Source/Track/DispatchLogic.rs
// Role: Main dispatch functions for routing commands and RPC requests.
// Responsibilities:
//   - Route all incoming requests to the appropriate execution logic via the
//     effect system.
//   - Provide specific, high-performance Tauri command handlers for frequent UI
//     interactions.

//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//!
//! # DispatchLogic
//!
//! Contains the main dispatch functions for routing all incoming commands and
//! RPC requests to the appropriate execution logic via the effect system.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{Environment::Requires::Requires, IPC::IPCProvider::IPCProvider};
use log::{debug, error, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime, State, command};

use crate::{
	ApplicationState::ApplicationState::ApplicationState,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track::EffectCreation,
};

/// The primary Tauri command handler for requests originating from the `Sky`
/// frontend. This is the general-purpose entry point for commands that are
/// defined abstractly in the `Common` crate.
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

		Err(Error) => {
			error!("[DispatchLogic] Failed to create effect for command '{}': {}", Command, Error);

			Err(Error)
		},
	}
}

/// The primary dispatcher for requests originating from a `Cocoon` sidecar via
/// gRPC. This routes RPC calls to the correct effect-based implementation.
pub async fn DispatchSideCarRequest<R:Runtime>(
	ApplicationHandle:AppHandle<R>,

	RunTime:Arc<ApplicationRunTime>,

	SideCarIdentifier:String,

	MethodName:String,

	Parameters:Value,
) -> Result<Value, String> {
	debug!(
		"[DispatchLogic] Dispatching sidecar request from '{}': {}",
		SideCarIdentifier, MethodName
	);

	match EffectCreation::CreateEffectForRequest(&ApplicationHandle, &MethodName, Parameters) {
		Ok(EffectFn) => EffectFn(RunTime).await,

		Err(Error) => {
			error!(
				"[DispatchLogic] Failed to create effect for sidecar method '{}': {}",
				MethodName, Error
			);

			Err(Error)
		},
	}
}

/// A specific Tauri command handler for the UI to send back the result of a
/// request-response interaction (like a dialog or message box).
#[command]
pub async fn ResolveUIRequest(
	State:State<'_, Arc<ApplicationState>>,

	RequestID:String,

	Result:Value,
) -> Result<(), String> {
	debug!("[DispatchLogic] Resolving UI request ID: {}", RequestID);

	let Sender = {
		let mut PendingRequests = State.PendingUserInterfaceRequests.lock().map_err(|Error| Error.to_string())?;

		PendingRequests.remove(&RequestID)
	};

	if let Some(Sender) = Sender {
		if Sender.send(Ok(Result)).is_err() {
			let ErrorMessage = format!("Failed to send result for UI request '{}': receiver was dropped.", RequestID);

			error!("{}", ErrorMessage);

			return Err(ErrorMessage);
		}
	} else {
		warn!(
			"[DispatchLogic] Received a result for an unknown or timed-out UI request ID: {}",
			RequestID
		);
	}

	Ok(())
}

/// A specific Tauri command handler for a WebView guest to post a message back
/// to the extension host.
#[command]
pub async fn MountainWebviewPostMessageFromGuest(
	ApplicationHandle:AppHandle,

	Handle:String,

	Message:Value,
) -> Result<(), String> {
	let IPC:Arc<dyn IPCProvider> = {
		let RunTime = ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

		RunTime.Environment.Require()
	};

	let RPCResult = IPC
		.SendNotificationToSideCar("cocoon-main".into(), "$onDidReceiveMessage".into(), json!([Handle, Message]))
		.await;

	if let Err(Error) = RPCResult {
		error!("[DispatchLogic] Failed to forward webview message to Cocoon: {}", Error);

		return Err(Error.to_string());
	}

	Ok(())
}
