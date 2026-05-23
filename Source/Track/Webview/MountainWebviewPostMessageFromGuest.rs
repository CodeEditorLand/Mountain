
//! # MountainWebviewPostMessageFromGuest (Track)
//!
//! ## RESPONSIBILITIES
//!
//! This module provides a Tauri command handler for a Webview guest to post
//! a message back to the extension host.
//!
//! ### Core Functions:
//! - Get IPC provider from runtime
//! - Forward message to main Cocoon sidecar
//! - Handle IPC errors gracefully
//!
//! ## ARCHITECTURAL ROLE
//!
//! MountainWebviewPostMessageFromGuest acts as the **webview message
//! forwarder** in Track's dispatch layer:
//!
//! ```text
//! Webview (Guest) ──► MountainWebviewPostMessageFromGuest ──► IPC Provider ──► Cocoon (Sidecar)
//! ```
//!
//! ## KEY COMPONENTS
//!
//! - **Fn**: Main webview message forwarding function (public async fn Fn)
//!
//! ## ERROR HANDLING
//!
//! - IPC communication errors are logged and propagated to caller
//! - Provider requirement failures are propagated
//!
//! ## LOGGING
//!
//! - Message forwarding failures are logged at error level
//! - Log format: "[Track/Webview] Forwarding webview message to Cocoon"
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Direct IPC provider access without intermediate overhead
//! - Async IPC operations to avoid blocking
//!
//! ## TODO
//!
//! - [ ] Add message validation before forwarding
//! - [ ] Implement message rate limiting
//! - [ ] Add message metrics and telemetry

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, IPC::IPCProvider::IPCProvider};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, command};

use crate::{
	ApplicationState::State::ApplicationState::ApplicationState,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// A specific Tauri command handler for a Webview guest to post a message back
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
		dev_log!(
			"ipc",
			"error: [Track/Webview] Failed to forward webview message to Cocoon: {}",
			Error
		);

		return Err(Error.to_string());
	}

	Ok(())
}
