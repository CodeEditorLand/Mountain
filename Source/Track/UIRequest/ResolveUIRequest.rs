//! # ResolveUIRequest (Track)
//!
//! ## RESPONSIBILITIES
//!
//! This module provides a Tauri command handler for the UI to send back the
//! result of a request-response interaction (like a dialog or message box).
//!
//! ### Core Functions:
//! - Look up pending UI request by ID
//! - Remove request from pending requests map
//! - Send result through oneshot channel
//! - Handle dropped receiver errors gracefully
//!
//! ## ARCHITECTURAL ROLE
//!
//! ResolveUIRequest acts as the **UI response handler** in Track's dispatch
//! layer:
//!
//! ```text
//! UI (Frontend) ──► ResolveUIRequest ──► Complete Pending Request ──► Return Result
//! ```
//!
//! ## KEY COMPONENTS
//!
//! - **Fn**: Main UI request resolution function (public async fn Fn)
//!
//! ## ERROR HANDLING
//!
//! - Unknown or timed-out requests are logged as warnings
//! - Dropped receiver errors are logged and returned as errors
//! - Lock errors on pending requests map are propagated
//!
//! ## LOGGING
//!
//! - All UI request resolutions are logged at debug level
//! - Unknown/timed-out requests are logged at warn level
//! - Dropped receiver errors are logged at error level
//! - Log format: "[Track/UIRequest] Resolving UI request ID: {}"
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Minimal locking duration on pending requests map
//! - Fast lookup and removal from HashMap
//! - Non-blocking channel sends
//!
//! ## TODO
//!
//! - [ ] Add request timeout handling
//! - [ ] Implement request cancellation support (VS Code compatibility)
//! - [ ] Add request metrics and telemetry

use std::sync::Arc;

use log::{debug, error, warn};
use serde_json::Value;
use tauri::{State, command};

use crate::ApplicationState::ApplicationState;

/// A specific Tauri command handler for the UI to send back the result of a
/// request-response interaction (like a dialog or message box).
#[command]
pub async fn ResolveUIRequest(
	State:State<'_, Arc<ApplicationState>>,

	RequestID:String,

	Result:Value,
) -> Result<(), String> {
	debug!("[Track/UIRequest] Resolving UI request ID: {}", RequestID);

	let Sender = {
	let mut PendingRequests = State
	.UI
	.PendingUserInterfaceRequest
	.lock()
	.map_err(|Error| Error.to_string())?;

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
			"[Track/UIRequest] Received a result for an unknown or timed-out UI request ID: {}",
			RequestID
		);
	}

	Ok(())
}
