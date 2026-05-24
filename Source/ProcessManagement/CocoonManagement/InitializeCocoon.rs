//! `CocoonManagement::InitializeCocoon`

use std::{collections::HashMap, process::Stdio, sync::Arc, time::Duration};
use CommonLibrary::Error::CommonError::CommonError;
use tauri::{
	AppHandle,
	Manager,
	Wry,
	path::{BaseDirectory, PathResolver},
};
use tokio::{
	io::{AsyncBufReadExt, BufReader},
	process::{Child, Command},
	sync::Mutex,
	time::sleep,
};
use super::{InitializationData, NodeResolver};
use crate::{
	Environment::MountainEnvironment::MountainEnvironment,
	IPC::Common::HealthStatus::{HealthIssue::Enum as HealthIssue, HealthMonitor::Struct as HealthMonitor},
	ProcessManagement::ExtractDevTag::Fn as ExtractDevTag,
	Vine,
	dev_log,
};

const COCOON_SIDE_CAR_IDENTIFIER:&str = "cocoon-main";
const COCOON_GRPC_PORT:u16 = 50052;
const MOUNTAIN_GRPC_PORT:u16 = 50051;
const BOOTSTRAP_SCRIPT_PATH:&str = "scripts/cocoon/bootstrap-fork.js";
const GRPC_CONNECT_INITIAL_MS:u64 = 50;
const GRPC_CONNECT_MAX_DELAY_MS:u64 = 2_000;
const GRPC_CONNECT_BUDGET_MS:u64 = 30_000;
const COCOON_BUNDLE_PROBE:&str = "../Cocoon/Target/Bootstrap/Implementation/Cocoon/Main.js";
const HANDSHAKE_TIMEOUT_MS:u64 = 60000;
const HEALTH_CHECK_INTERVAL_SECONDS:u64 = 5;
const MAX_RESTART_ATTEMPTS:u32 = 3;
const RESTART_WINDOW_SECONDS:u64 = 300;
static COCOON_PID:std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// The main entry point for initializing the Cocoon sidecar process manager.
///
/// This orchestrates the complete initialization sequence including:
/// - Validating feature flags and dependencies
/// - Launching the Cocoon process with proper configuration
/// - Establishing gRPC communication
/// - Performing the initialization handshake
/// - Setting up process health monitoring
///
/// # Arguments
///
/// * `ApplicationHandle` - Tauri application handle for path resolution
/// * `Environment` - Mountain environment containing application state and
///   services
///
/// # Returns
///
/// * `Ok(())` - Cocoon initialized successfully and ready to accept extension
///   requests
/// * `Err(CommonError)` - Initialization failed with detailed error context
///
/// # Errors
///
/// - `FileSystemNotFound`: Bootstrap script not found
/// - `IPCError`: Failed to spawn process or establish gRPC connection
///
/// # Example
///
/// ```rust,no_run
/// use crate::Source::ProcessManagement::CocoonManagement::InitializeCocoon;
///
/// InitializeCocoon(&app_handle, &environment).await?;
/// ```
pub async fn Fn(
	ApplicationHandle:&AppHandle,

	Environment:&Arc<MountainEnvironment>,
) -> Result<(), CommonError> {
	dev_log!("cocoon", "[CocoonManagement] Initializing Cocoon sidecar manager...");

	// Atom N1: `debug-mountain-only` / `release-mountain-only` profiles set
	// Spawn=false so Mountain boots without the extension host.
	// Extension-related IPC returns the empty-state envelope; the workbench
	// loads but no extension activates. Useful for integration tests that
	// exercise Mountain in isolation and for the smallest shippable surface.
	if matches!(std::env::var("Spawn").as_deref(), Ok("0") | Ok("false")) {
		dev_log!("cocoon", "[CocoonManagement] Skipping spawn (Spawn=false)");

		return Ok(());
	}

	#[cfg(feature = "ExtensionHostCocoon")]
	{
		LaunchAndManageCocoonSideCar(ApplicationHandle.clone(), Environment.clone()).await
	}

	#[cfg(not(feature = "ExtensionHostCocoon"))]
	{
		dev_log!(
			"cocoon",
			"[CocoonManagement] 'ExtensionHostCocoon' feature is disabled. Cocoon will not be launched."
		);

		Ok(())
	}
}
