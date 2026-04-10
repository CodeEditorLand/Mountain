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

use std::{collections::HashMap, process::Stdio, sync::Arc, time::Duration};

use CommonLibrary::Error::CommonError::CommonError;
use log::{info, trace, warn};
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

use super::InitializationData;
use crate::{
	Environment::MountainEnvironment::MountainEnvironment,
	IPC::Common::HealthStatus::{HealthIssue, HealthMonitor},
	Vine,
};

/// Configuration constants for Cocoon process management
const COCOON_SIDE_CAR_IDENTIFIER:&str = "cocoon-main";
const COCOON_GRPC_PORT:u16 = 50052;
const MOUNTAIN_GRPC_PORT:u16 = 50051;
const GRPC_SERVER_READY_DELAY_MS:u64 = 2000;
const BOOTSTRAP_SCRIPT_PATH:&str = "scripts/cocoon/bootstrap-fork.js";
const HANDSHAKE_TIMEOUT_MS:u64 = 60000;
const HEALTH_CHECK_INTERVAL_SECONDS:u64 = 5;
const MAX_RESTART_ATTEMPTS:u32 = 3;
const RESTART_WINDOW_SECONDS:u64 = 300;

/// Global state for tracking Cocoon process lifecycle
struct CocoonProcessState {
	ChildProcess:Option<Child>,
	IsRunning:bool,
	StartTime:Option<tokio::time::Instant>,
	RestartCount:u32,
	LastRestartTime:Option<tokio::time::Instant>,
}

impl Default for CocoonProcessState {
	fn default() -> Self {
		Self {
			ChildProcess:None,
			IsRunning:false,
			StartTime:None,
			RestartCount:0,
			LastRestartTime:None,
		}
	}
}

// Global state for Cocoon process management
lazy_static::lazy_static! {
	static ref COCOON_STATE: Arc<Mutex<CocoonProcessState>> =
		Arc::new(Mutex::new(CocoonProcessState::default()));

	static ref COCOON_HEALTH: Arc<Mutex<HealthMonitor>> =
		Arc::new(Mutex::new(HealthMonitor::new()));
}

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
pub async fn InitializeCocoon(
	ApplicationHandle:&AppHandle,
	Environment:&Arc<MountainEnvironment>,
) -> Result<(), CommonError> {
	info!("[CocoonManagement] Initializing Cocoon sidecar manager...");

	#[cfg(feature = "ExtensionHostCocoon")]
	{
		LaunchAndManageCocoonSideCar(ApplicationHandle.clone(), Environment.clone()).await
	}

	#[cfg(not(feature = "ExtensionHostCocoon"))]
	{
		info!("[CocoonManagement] 'ExtensionHostCocoon' feature is disabled. Cocoon will not be launched.");
		Ok(())
	}
}

/// Spawns the Cocoon process, manages its communication channels, and performs
/// the complete initialization handshake sequence.
///
/// This function implements the complete Cocoon lifecycle:
/// 1. Validates bootstrap script availability
/// 2. Constructs environment variables for IPC and logging
/// 3. Spawns Node.js process with proper IO redirection
/// 4. Captures stdout/stderr for logging
/// 5. Waits for gRPC server to be ready
/// 6. Establishes Vine connection
/// 7. Sends initialization payload and validates response
///
/// # Arguments
///
/// * `ApplicationHandle` - Tauri application handle for resolving resource
///   paths
/// * `Environment` - Mountain environment containing application state
///
/// # Returns
///
/// * `Ok(())` - Cocoon process spawned, connected, and initialized successfully
/// * `Err(CommonError)` - Any failure during the initialization sequence
///
/// # Errors
///
/// - `FileSystemNotFound`: Bootstrap script not found in resources
/// - `IPCError`: Failed to spawn process, connect gRPC, or complete handshake
///
/// # Lifecycle
///
/// The process runs as a background task with IO redirection for logging.
/// Process failures are logged but not automatically restarted (callers should
/// implement restart strategies based on their requirements).
async fn LaunchAndManageCocoonSideCar(
	ApplicationHandle:AppHandle,
	Environment:Arc<MountainEnvironment>,
) -> Result<(), CommonError> {
	let SideCarIdentifier = COCOON_SIDE_CAR_IDENTIFIER.to_string();
	let path_resolver:PathResolver<Wry> = ApplicationHandle.path().clone();

	// Resolve bootstrap script path.
	// 1) Try Tauri bundled resources (production builds).
	// 2) Fallback: resolve relative to the executable (dev builds).
	//    Dev layout: Target/debug/binary → ../../scripts/cocoon/bootstrap-fork.js
	let ScriptPath = path_resolver
		.resolve(BOOTSTRAP_SCRIPT_PATH, BaseDirectory::Resource)
		.ok()
		.filter(|P| P.exists())
		.or_else(|| {
			std::env::current_exe().ok().and_then(|Exe| {
				let MountainRoot = Exe.parent()?.parent()?.parent()?;
				let Candidate = MountainRoot.join(BOOTSTRAP_SCRIPT_PATH);
				if Candidate.exists() { Some(Candidate) } else { None }
			})
		})
		.ok_or_else(|| {
			CommonError::FileSystemNotFound(
				format!("Cocoon bootstrap script '{}' not found in resources or relative to executable", BOOTSTRAP_SCRIPT_PATH).into(),
			)
		})?;

	info!("[CocoonManagement] Found bootstrap script at: {}", ScriptPath.display());
	crate::dev_log!("cocoon", "bootstrap script: {}", ScriptPath.display());

	// Build Node.js command with comprehensive environment configuration
	let mut NodeCommand = Command::new("node");

	let mut EnvironmentVariables = HashMap::new();

	// VS Code protocol environment variables for extension host compatibility
	EnvironmentVariables.insert("VSCODE_PIPE_LOGGING".to_string(), "true".to_string());
	EnvironmentVariables.insert("VSCODE_VERBOSE_LOGGING".to_string(), "true".to_string());
	EnvironmentVariables.insert("VSCODE_PARENT_PID".to_string(), std::process::id().to_string());

	// gRPC port configuration for Vine communication
	EnvironmentVariables.insert("MOUNTAIN_GRPC_PORT".to_string(), MOUNTAIN_GRPC_PORT.to_string());
	EnvironmentVariables.insert("COCOON_GRPC_PORT".to_string(), COCOON_GRPC_PORT.to_string());

	// Preserve PATH so `node` resolves. env_clear() was stripping it.
	if let Ok(Path) = std::env::var("PATH") {
		EnvironmentVariables.insert("PATH".to_string(), Path);
	}
	if let Ok(Home) = std::env::var("HOME") {
		EnvironmentVariables.insert("HOME".to_string(), Home);
	}

	NodeCommand
		.arg(&ScriptPath)
		.env_clear()
		.envs(EnvironmentVariables)
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped());

	// Spawn the process with error handling
	let mut ChildProcess = NodeCommand.spawn().map_err(|Error| {
		CommonError::IPCError {
			Description:format!("Failed to spawn Cocoon process: {} (is Node.js installed and in PATH?)", Error),
		}
	})?;

	let ProcessId = ChildProcess.id().unwrap_or(0);
	info!("[CocoonManagement] Cocoon process spawned [PID: {}]", ProcessId);
	crate::dev_log!("cocoon", "spawned PID={}", ProcessId);

	// Capture stdout for trace logging
	if let Some(stdout) = ChildProcess.stdout.take() {
		tokio::spawn(async move {
			let Reader = BufReader::new(stdout);
			let mut Lines = Reader.lines();

			while let Ok(Some(Line)) = Lines.next_line().await {
				trace!("[Cocoon stdout] {}", Line);
			}
		});
	}

	// Capture stderr for warn-level logging
	if let Some(stderr) = ChildProcess.stderr.take() {
		tokio::spawn(async move {
			let Reader = BufReader::new(stderr);
			let mut Lines = Reader.lines();

			while let Ok(Some(Line)) = Lines.next_line().await {
				warn!("[Cocoon stderr] {}", Line);
			}
		});
	}

	// Wait for gRPC server to initialize and listen
	info!(
		"[CocoonManagement] Waiting {}ms for Cocoon gRPC server to start...",
		GRPC_SERVER_READY_DELAY_MS
	);
	sleep(Duration::from_millis(GRPC_SERVER_READY_DELAY_MS)).await;

	// Establish Vine connection to Cocoon
	let GRPCAddress = format!("127.0.0.1:{}", COCOON_GRPC_PORT);
	info!("[CocoonManagement] Connecting to Cocoon gRPC server at: {}", GRPCAddress);
	crate::dev_log!("grpc", "connecting to Cocoon at {}", GRPCAddress);

	Vine::Client::ConnectToSideCar(SideCarIdentifier.clone(), GRPCAddress.clone())
		.await
		.map_err(|Error| {
			CommonError::IPCError {
				Description:format!(
					"Failed to connect to Cocoon gRPC server at {}: {} (is Cocoon running?)",
					GRPCAddress, Error
				),
			}
		})?;

	info!("[CocoonManagement] Connected to Cocoon. Sending initialization data...");

	// Construct initialization payload
	let MainInitializationData = InitializationData::ConstructExtensionHostInitializationData(&Environment)
		.await
		.map_err(|Error| {
			CommonError::IPCError { Description:format!("Failed to construct initialization data: {}", Error) }
		})?;

	// Send initialization request with timeout
	let Response = Vine::Client::SendRequest(
		&SideCarIdentifier,
		"InitializeExtensionHost".to_string(),
		MainInitializationData,
		HANDSHAKE_TIMEOUT_MS,
	)
	.await
	.map_err(|Error| {
		CommonError::IPCError {
			Description:format!("Failed to send initialization request to Cocoon: {}", Error),
		}
	})?;

	// Validate handshake response
	match Response.as_str() {
		Some("initialized") => {
			info!("[CocoonManagement] Cocoon handshake complete. Extension host is ready.");
		},
		Some(other) => {
			return Err(CommonError::IPCError {
				Description:format!("Cocoon initialization failed with unexpected response: {}", other),
			});
		},
		None => {
			return Err(CommonError::IPCError {
				Description:"Cocoon initialization failed: no response received".to_string(),
			});
		},
	}

	// Store process handle for health monitoring and management
	{
		let mut state = COCOON_STATE.lock().await;
		state.ChildProcess = Some(ChildProcess);
		state.IsRunning = true;
		state.StartTime = Some(tokio::time::Instant::now());
		info!("[CocoonManagement] Process state updated: Running");
	}

	// Reset health monitor on successful initialization
	{
		let mut health = COCOON_HEALTH.lock().await;
		health.clear_issues();
		info!("[CocoonManagement] Health monitor reset to active state");
	}

	// Start background health monitoring
	let state_clone = Arc::clone(&COCOON_STATE);
	tokio::spawn(monitor_cocoon_health_task(state_clone));
	info!("[CocoonManagement] Background health monitoring started");

	Ok(())
}

/// Background task that monitors Cocoon process health and logs crashes
async fn monitor_cocoon_health_task(state:Arc<Mutex<CocoonProcessState>>) {
	loop {
		tokio::time::sleep(Duration::from_secs(HEALTH_CHECK_INTERVAL_SECONDS)).await;

		let mut state_guard = state.lock().await;

		// Check if we have a child process to monitor
		if state_guard.ChildProcess.is_some() {
			// Get process ID before checking status
			let process_id = state_guard.ChildProcess.as_ref().map(|c| c.id().unwrap_or(0));

			// Check if process is still running
			let exit_status = {
				let child = state_guard.ChildProcess.as_mut().unwrap();
				child.try_wait()
			};

			match exit_status {
				Ok(Some(exit_code)) => {
					// Process has exited (crashed or terminated)
					let uptime = state_guard.StartTime.map(|t| t.elapsed().as_secs()).unwrap_or(0);
					let exit_code_num = exit_code.code().unwrap_or(-1);
					warn!(
						"[CocoonHealth] Cocoon process crashed [PID: {}] [Exit Code: {}] [Uptime: {}s]",
						process_id.unwrap_or(0),
						exit_code_num,
						uptime
					);

					// Update state
					state_guard.IsRunning = false;
					state_guard.ChildProcess = None;

					// Report health issue
					{
						let mut health = COCOON_HEALTH.lock().await;
						health.add_issue(HealthIssue::Custom(format!("ProcessCrashed (Exit code: {})", exit_code_num)));
						warn!("[CocoonHealth] Health score: {}", health.health_score);
					}

					// Log that automatic restart would be needed
					warn!(
						"[CocoonHealth] CRASH DETECTED: Cocoon process has crashed and must be restarted manually or \
						 via application reinitialization"
					);
				},
				Ok(None) => {
					// Process is still running
					trace!("[CocoonHealth] Cocoon process is healthy [PID: {}]", process_id.unwrap_or(0));
				},
				Err(e) => {
					// Error checking process status
					warn!("[CocoonHealth] Error checking process status: {}", e);

					// Report health issue
					{
						let mut health = COCOON_HEALTH.lock().await;
						health.add_issue(HealthIssue::Custom(format!("ProcessCheckError: {}", e)));
					}
				},
			}
		} else {
			// No child process exists
			trace!("[CocoonHealth] No Cocoon process to monitor");
		}
	}
}
