#![allow(unused_imports)]

//! # DispatchFrontendCommand (Track)
//!
//! ## RESPONSIBILITIES
//!
//! This module provides the primary Tauri command handler for requests
//! originating from the Sky frontend. It serves as the general-purpose entry
//! point for commands that are defined abstractly in the Common crate.
//!
//! ### Core Functions:
//! - Receive frontend commands via Tauri IPC
//! - Route commands to the effect creation system
//! - Execute effects through the ApplicationRunTime
//! - Return results or errors to the frontend
//!
//! ## ARCHITECTURAL ROLE
//!
//! DispatchFrontendCommand acts as the **frontend gateway** in Track's dispatch
//! layer:
//!
//! ```text
//! Sky (Frontend) ──► DispatchFrontendCommand ──► CreateEffectForRequest ──► ApplicationRunTime ──► Providers
//! ```
//!
//! ## KEY COMPONENTS
//!
//! - **Fn**: Main dispatch function (public async fn Fn<R:Runtime>)
//!
//! ## ERROR HANDLING
//!
//! - Effect creation failures are caught and logged
//! - Unknown commands are reported with context
//! - Errors are propagated to the frontend with descriptive messages
//!
//! ## LOGGING
//!
//! - All incoming commands are logged at debug level
//! - Effect creation failures are logged at error level
//! - Log format: "[Track/FrontendCommand] Dispatching frontend command: {}"
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Direct effect execution without intermediate overhead
//! - Minimal locking to avoid blocking the UI thread
//! - Async operations to prevent blocking
//!
//! ## TODO
//!
//! - [ ] Add request timeout handling
//! - [ ] Implement request cancellation support (VS Code compatibility)
//! - [ ] Add request metrics and telemetry

use std::sync::Arc;

use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime, State, command};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::CreateEffectForRequest};
use crate::dev_log;

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
	dev_log!("ipc", "[Track/FrontendCommand] Dispatching frontend command: {}", Command);

	match CreateEffectForRequest(&ApplicationHandle, &Command, Argument) {
		Ok(EffectFn) => {
			let runtime_clone = RunTime.inner().clone();

			EffectFn(runtime_clone).await
		},

		Err(Error) => {
			dev_log!("ipc", "error: [Track/FrontendCommand] Failed to create effect for command '{}': {}",
				Command, Error);

			Err(Error)
		},
	}
}
