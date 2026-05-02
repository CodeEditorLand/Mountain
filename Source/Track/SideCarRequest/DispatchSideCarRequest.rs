//! # DispatchSideCarRequest (Track)
//!
//! ## RESPONSIBILITIES
//!
//! This module provides the primary dispatcher for requests originating from
//! Cocoon sidecars via gRPC. It routes RPC calls to the correct effect-based
//! implementation.
//!
//! ### Core Functions:
//! - Receive gRPC requests from Cocoon sidecars
//! - Route requests to the effect creation system
//! - Execute effects through the ApplicationRunTime
//! - Return results or errors to the sidecar
//!
//! ## ARCHITECTURAL ROLE
//!
//! DispatchSideCarRequest acts as the **sidecar gateway** in Track's dispatch
//! layer:
//!
//! ```text
//! Cocoon (Sidecar) ──► DispatchSideCarRequest ──► CreateEffectForRequest ──► ApplicationRunTime ──► Providers
//! ```
//!
//! ## KEY COMPONENTS
//!
//! - **Fn**: Main dispatch function (public async fn Fn<R:Runtime>)
//!
//! ## ERROR HANDLING
//!
//! - Effect creation failures are caught and logged
//! - Unknown methods are reported with context
//! - Errors are propagated to the sidecar with descriptive messages
//!
//! ## LOGGING
//!
//! - All incoming sidecar requests are logged at debug level with sidecar ID
//! - Effect creation failures are logged at error level
//! - Log format: "[Track/SideCarRequest] Dispatching sidecar request from '{}':
//!   {}"
//!
//! ## PERFORMANCE CONSIDERATIONS
//!
//! - Direct effect execution without intermediate overhead
//! - Minimal locking to avoid blocking
//! - Async operations for non-blocking dispatch
//!
//! ## TODO
//!
//! - [ ] Add request timeout handling
//! - [ ] Implement request cancellation support (VS Code compatibility)
//! - [ ] Add request metrics and telemetry
//! - [ ] Add sidecar authentication/authorization

use std::sync::Arc;

use serde_json::Value;
use tauri::{AppHandle, Runtime};

use crate::{
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track::Effect::CreateEffectForRequest::Fn as CreateEffectForRequest,
	dev_log,
};

/// The primary dispatcher for requests originating from a `Cocoon` sidecar via
/// gRPC. This routes RPC calls to the correct effect-based implementation.
pub async fn DispatchSideCarRequest<R:Runtime>(
	ApplicationHandle:AppHandle<R>,

	RunTime:Arc<ApplicationRunTime>,

	SideCarIdentifier:String,

	MethodName:String,

	Parameters:Value,
) -> Result<Value, String> {
	// Per-request dispatch line - fires for every FileSystem.ReadFile /
	// FileSystem.Stat / Configuration.Inspect round-trip from Cocoon. The
	// caller-side `[DEV:IPC] invoke:` and `done:` pair already carries the
	// method + timing (when not in the high-frequency skip list), so this
	// line adds nothing at the default log level. Route to `grpc-verbose`.
	dev_log!(
		"grpc-verbose",
		"[Track/SideCarRequest] Dispatching sidecar request from '{}': {}",
		SideCarIdentifier,
		MethodName
	);

	match CreateEffectForRequest(&ApplicationHandle, &MethodName, Parameters) {
		Ok(EffectFn) => EffectFn(RunTime).await,

		Err(Error) => {
			dev_log!(
				"ipc",
				"error: [Track/SideCarRequest] Failed to create effect for sidecar method '{}': {}",
				MethodName,
				Error
			);

			Err(Error)
		},
	}
}
