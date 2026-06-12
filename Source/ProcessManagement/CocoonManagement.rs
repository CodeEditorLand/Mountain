//! # Cocoon Management
//!
//! This module provides comprehensive lifecycle management for the Cocoon
//! sidecar process, which serves as the VS Code extension host within the
//! Mountain editor.
//!
//! ## Overview
//!
//! Cocoon is a Node.js-based process that provides compatibility with VS Code
//! extensions. This module handles:
//!
//! - **Process Spawning**: Launching Node.js with the Cocoon bootstrap script
//! - **Environment Configuration**: Setting up environment variables for IPC
//!   and logging
//! - **Communication Setup**: Establishing gRPC/Vine connections on port 50052
//! - **Health Monitoring**: Tracking process state and handling failures
//! - **Lifecycle Management**: Graceful shutdown and restart capabilities
//! - **IO Redirection**: Capturing stdout/stderr for logging and debugging
//!
//! ## Process Communication
//!
//! The Cocoon process communicates via:
//! - gRPC on port 50052 (configured via MOUNTAIN_GRPC_PORT/COCOON_GRPC_PORT)
//! - Vine protocol for cross-process messaging
//! - Standard streams for logging (VSCODE_PIPE_LOGGING)
//!
//! ## Dependencies
//!
//! - `scripts/cocoon/bootstrap-fork.js`: Bootstrap script for launching Cocoon
//! - Node.js runtime: Required for executing Cocoon
//! - Vine gRPC server: Must be running on port 50051 for handshake
//!
//! ## Error Handling
//!
//! The module provides graceful degradation:
//! - If the bootstrap script is missing, returns `FileSystemNotFound` error
//! - If Node.js cannot be spawned, returns `IPCError`
//! - If gRPC connection fails, returns `IPCError` with context
//!
//! # Module Contents
//!
//! - [`InitializeCocoon`]: Main entry point for Cocoon initialization
//! - `LaunchAndManageCocoonSideCar`: Process spawning and lifecycle
//! management
//!
//! ## Example
//!
//! ```rust,no_run
//! use crate::Source::ProcessManagement::CocoonManagement::InitializeCocoon;
//!
//! // Initialize Cocoon with application handle and environment
//! match InitializeCocoon(&app_handle, &environment).await {
//! 	Ok(()) => println!("Cocoon initialized successfully"),
//! 	Err(e) => eprintln!("Cocoon initialization failed: {:?}", e),
//! }
//! ```

use std::sync::{Arc, OnceLock};

use once_cell::sync::Lazy;
use CommonLibrary::Error::CommonError::CommonError;
use tauri::AppHandle;
use tokio::sync::Mutex;

use crate::{
	Environment::MountainEnvironment::MountainEnvironment,
	IPC::Common::HealthStatus::HealthMonitor::Struct as HealthMonitor,
	dev_log,
};

#[path = "CocoonManagement/BuildCocoonEnvironment.rs"]
/// Buildcocoonenvironment module.
pub mod BuildCocoonEnvironment;

#[path = "CocoonManagement/CocoonProcessState.rs"]
/// Cocoonprocessstate module.
pub mod CocoonProcessState;

#[path = "CocoonManagement/ConnectToCocoonGrpc.rs"]
/// Connecttococoongrpc module.
pub mod ConnectToCocoonGrpc;

#[path = "CocoonManagement/FindMatchingWorkspaceContainsPatterns.rs"]
/// Findmatchingworkspacecontainspatterns module.
pub mod FindMatchingWorkspaceContainsPatterns;

#[path = "CocoonManagement/FireRootConfigActivationEvents.rs"]
/// Firerootconfigactivationevents module.
pub mod FireRootConfigActivationEvents;

#[path = "CocoonManagement/FireWorkspaceContainsEvents.rs"]
/// Fireworkspacecontainsevents module.
pub mod FireWorkspaceContainsEvents;

#[path = "CocoonManagement/HardKillCocoon.rs"]
/// Hardkillcocoon module.
pub mod HardKillCocoon;

#[path = "CocoonManagement/LaunchAndManageCocoonSideCar.rs"]
/// Launchandmanagecocoonsidecar module.
pub mod LaunchAndManageCocoonSideCar;

#[path = "CocoonManagement/MonitorCocoonHealthTask.rs"]
/// Monitorcocoonhealthtask module.
pub mod MonitorCocoonHealthTask;

/// Patternmatchesanyentry module.
pub mod PatternMatchesAnyEntry;

/// Resolvebootstrapscript module.
pub mod ResolveBootstrapScript;

/// Restorewebviewpanels module.
pub mod RestoreWebviewPanels;

/// Seedopendocuments module.
pub mod SeedOpenDocuments;

/// Seedopenterminals module.
pub mod SeedOpenTerminals;

/// Segmentmatch module.
pub mod SegmentMatch;

/// Sendinitializationhandshake module.
pub mod SendInitializationHandshake;

/// Singlesegmentmatch module.
pub mod SingleSegmentMatch;

/// Spawncocoonioforwarders module.
pub mod SpawnCocoonIoForwarders;

/// Spawnrestarthandler module.
pub mod SpawnRestartHandler;

/// Spawnstartupactivationtask module.
pub mod SpawnStartupActivationTask;

/// Sweepstalecocoon module.
pub mod SweepStaleCocoon;

/// Configuration constants for Cocoon process management
const COCOON_SIDE_CAR_IDENTIFIER:&str = "cocoon-main";

const COCOON_GRPC_PORT:u16 = 50052;

// ============================================================================
// B7-S6: Per-spawn WebSocket configuration for Sky<->Cocoon direct transport.
// ============================================================================

static COCOON_WS_PORT_CELL:OnceLock<u16> = OnceLock::new();

static COCOON_WS_SECRET_CELL:OnceLock<String> = OnceLock::new();

fn InitializeWsConfig() {
	COCOON_WS_PORT_CELL.get_or_init(|| portpicker::pick_unused_port().unwrap_or(0));

	COCOON_WS_SECRET_CELL.get_or_init(|| {
		// Generate 32 random bytes as hex string.
		use std::fmt::Write;

		let Bytes:[u8; 32] = rand::random();

		Bytes.iter().fold(String::new(), |mut S, B| {
			write!(S, "{:02x}", B).ok();

			S
		})
	});
}

/// Wss port.
pub fn WsPort() -> u16 { *COCOON_WS_PORT_CELL.get().unwrap_or(&0) }

/// Wss secret hex.
pub fn WsSecretHex() -> String { COCOON_WS_SECRET_CELL.get().cloned().unwrap_or_default() }

const MOUNTAIN_GRPC_PORT:u16 = 50051;

const BOOTSTRAP_SCRIPT_PATH:&str = "scripts/cocoon/bootstrap-fork.js";

/// Exponential-backoff retry parameters for the Mountain → Cocoon gRPC
/// handshake. After the Bootstrap.ts stage-reorder fix, Cocoon's RPCServer
/// (port 50052) starts as Stage 3 (before MountainConnection), so the port
/// is available within 2-5 seconds of spawn. Budget raised to 30 s as a
/// defensive buffer for slow hardware or contended startup.
/// Policy: start at 50 ms, double each attempt up to a 2 s ceiling,
/// with a hard 30 s total-budget. Under healthy spawn timing (Cocoon
/// binds 50052 within 2-3s) this converges on attempts 5-8 in <~3s total;
/// under a genuinely dead Cocoon the loop abandons at the budget.
const GRPC_CONNECT_INITIAL_MS:u64 = 50;

const GRPC_CONNECT_MAX_DELAY_MS:u64 = 2_000;

const GRPC_CONNECT_BUDGET_MS:u64 = 30_000;

/// Relative path from the resolved Cocoon package root to the bundled
/// entry module. Used by the pre-flight guard below to fail fast with
/// an actionable error when the bundle is missing (esbuild failure,
/// partial rm -rf, freshly cloned checkout without `pnpm run
/// prepublishOnly`, etc.) instead of spawning Node into a dying
/// require() chain.
const COCOON_BUNDLE_PROBE:&str = "../Cocoon/Target/Bootstrap/Implementation/Cocoon/Main.js";

const HANDSHAKE_TIMEOUT_MS:u64 = 60000;

const HEALTH_CHECK_INTERVAL_SECONDS:u64 = 5;

const MAX_RESTART_ATTEMPTS:u32 = 3;

const RESTART_WINDOW_SECONDS:u64 = 300;

// Global state for Cocoon process management
static COCOON_STATE:Lazy<Arc<Mutex<CocoonProcessState::CocoonProcessState>>> =
	Lazy::new(|| Arc::new(Mutex::new(CocoonProcessState::CocoonProcessState::default())));

static COCOON_HEALTH:Lazy<Arc<Mutex<HealthMonitor>>> = Lazy::new(|| Arc::new(Mutex::new(HealthMonitor::new())));

/// Last-known PID of the Cocoon child process. Mirrored here so callers can
/// read it without taking the async `COCOON_STATE` mutex (e.g. from IPC
/// handlers such as `extensionHostStarter:start`). Set after spawn and
/// cleared on shutdown. `0` means "not running".
static COCOON_PID:std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Return the Cocoon child process's OS PID, or `None` if Cocoon has not
/// been spawned (or has exited).
pub fn GetCocoonPid() -> Option<u32> {
	match COCOON_PID.load(std::sync::atomic::Ordering::Relaxed) {
		0 => None,

		Pid => Some(Pid),
	}
}

/// The main entry point for initializing the Cocoon sidecar process manager.
/// This orchestrates the complete initialization sequence including:
/// - Validating feature flags and dependencies
/// - Launching the Cocoon process with proper configuration
/// - Establishing gRPC communication
/// - Performing the initialization handshake
/// - Setting up process health monitoring
/// # Arguments
/// * `ApplicationHandle` - Tauri application handle for path resolution
/// * `Environment` - Mountain environment containing application state and
///   services
/// # Returns
/// * `Ok(())` - Cocoon initialized successfully and ready to accept extension
///   requests
/// * `Err(CommonError)` - Initialization failed with detailed error context
/// # Errors
/// - `FileSystemNotFound`: Bootstrap script not found
/// - `IPCError`: Failed to spawn process or establish gRPC connection
/// # Example
/// ```rust,no_run
/// use crate::Source::ProcessManagement::CocoonManagement::InitializeCocoon;
/// InitializeCocoon(&app_handle, &environment).await?;
/// ```
pub async fn InitializeCocoon(
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

	#[cfg(all(feature = "ExtensionHostCocoon", not(no_node_host)))]
	{
		LaunchAndManageCocoonSideCar::Fn(ApplicationHandle.clone(), Environment.clone()).await
	}

	#[cfg(any(not(feature = "ExtensionHostCocoon"), no_node_host))]
	{
		dev_log!(
			"cocoon",
			"[CocoonManagement] Cocoon spawn gated off (feature=ExtensionHostCocoon disabled or \
			 TierExtensionHost=WebWorker)."
		);

		Ok(())
	}
}

/// Atom I6: post-shutdown hard-kill. Called by RuntimeShutdown after the
/// `$shutdown` gRPC notification has been sent (and either succeeded or
/// timed out). Grabs the stored `Child` handle and force-terminates it if
/// still alive, then resets COCOON_STATE. This plugs the "Mountain exits
/// cleanly but child stays running" leak that leads to zombie-Cocoon
/// zombies holding the gRPC port.
/// Call AFTER the graceful $shutdown attempt - we don't want to race the
/// child's own cleanup. Safe to call with no stored child (no-op).
pub async fn HardKillCocoon() { self::HardKillCocoon::Fn().await }
