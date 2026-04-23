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
	IPC::Common::HealthStatus::{HealthIssue, HealthMonitor},
	Vine,
	dev_log,
};

/// Configuration constants for Cocoon process management
const COCOON_SIDE_CAR_IDENTIFIER:&str = "cocoon-main";
const COCOON_GRPC_PORT:u16 = 50052;
const MOUNTAIN_GRPC_PORT:u16 = 50051;
const GRPC_CONNECT_RETRY_INTERVAL_MS:u64 = 1000;
const GRPC_CONNECT_MAX_ATTEMPTS:u32 = 20;
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
	dev_log!("cocoon", "[CocoonManagement] Initializing Cocoon sidecar manager...");

	// Atom N1: `debug-mountain-only` / `release-mountain-only` profiles set
	// LAND_SPAWN_COCOON=false so Mountain boots without the extension host.
	// Extension-related IPC returns the empty-state envelope; the workbench
	// loads but no extension activates. Useful for integration tests that
	// exercise Mountain in isolation and for the smallest shippable surface.
	if matches!(std::env::var("LAND_SPAWN_COCOON").as_deref(), Ok("0") | Ok("false")) {
		dev_log!("cocoon", "[CocoonManagement] Skipping spawn (LAND_SPAWN_COCOON=false)");
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
	// 2) Fallback: resolve relative to the executable (dev builds). Dev layout:
	//    Target/debug/binary → ../../scripts/cocoon/bootstrap-fork.js
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
				format!(
					"Cocoon bootstrap script '{}' not found in resources or relative to executable",
					BOOTSTRAP_SCRIPT_PATH
				)
				.into(),
			)
		})?;

	dev_log!(
		"cocoon",
		"[CocoonManagement] Found bootstrap script at: {}",
		ScriptPath.display()
	);
	crate::dev_log!("cocoon", "bootstrap script: {}", ScriptPath.display());

	// Atom I6: zombie-Cocoon sweep. If a prior Mountain exited without
	// killing its child (segfault, SIGKILL, debugger detach, …), the stale
	// node process keeps port COCOON_GRPC_PORT bound. The new Mountain's
	// VineClient then "successfully connects" to the zombie while the
	// freshly-spawned Cocoon fails to bind with EADDRINUSE, and the whole
	// extension host enters degraded mode with zero extensions visible.
	//
	// Probe the port. If it answers, find the owning PID via `lsof -t -i
	// :<port>` and SIGTERM → 500ms wait → SIGKILL. Then proceed as normal.
	SweepStaleCocoon(COCOON_GRPC_PORT);

	// Atom N1: resolve Node binary via NodeResolver (shipped → version
	// managers → homebrew → PATH). Logs the pick + source for forensics.
	// Overridable via `LAND_NODE_BINARY=/absolute/path/to/node`.
	let ResolvedNodeBinary = NodeResolver::ResolveNodeBinary(&ApplicationHandle);

	// Build Node.js command with comprehensive environment configuration
	let mut NodeCommand = Command::new(&ResolvedNodeBinary.Path);

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

	// Atom I5: forward every Product*, Tier*, Network* env var from
	// .env.Land into the Cocoon subprocess. Cocoon's InitData.ts +
	// ExtensionHostHandler.ts read these at startup for version,
	// identity, and port configuration. Without this forwarding, the
	// whitelist above drops them and Cocoon falls back to defaults,
	// defeating the single-source-of-truth design.
	for (Key, Value) in std::env::vars() {
		if Key.starts_with("Product") || Key.starts_with("Tier") || Key.starts_with("Network") {
			EnvironmentVariables.insert(Key, Value);
		}
	}

	// Atom I11: forward NODE_ENV / LAND_DEV_LOG / TAURI_ENV_DEBUG so
	// Cocoon's Bootstrap.ts stage2_configuration resolves real values.
	// Without this, env_clear() above leaves Cocoon seeing NodeEnv=
	// "production" / DevLog=<unset> / TauriDebug=false even on the
	// debug-electron profile - silently disabling dev-only logging,
	// stricter validation, and debug-only diagnostics in Cocoon.
	for Key in ["NODE_ENV", "LAND_DEV_LOG", "TAURI_ENV_DEBUG"] {
		if let Ok(Value) = std::env::var(Key) {
			EnvironmentVariables.insert(Key.to_string(), Value);
		}
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
			Description:format!(
				"Failed to spawn Cocoon with node={} (source={}): {}. Override with \
				 LAND_NODE_BINARY=/absolute/path or install Node.js.",
				ResolvedNodeBinary.Path.display(),
				ResolvedNodeBinary.Source.AsLabel(),
				Error
			),
		}
	})?;

	let ProcessId = ChildProcess.id().unwrap_or(0);
	COCOON_PID.store(ProcessId, std::sync::atomic::Ordering::Relaxed);
	dev_log!("cocoon", "[CocoonManagement] Cocoon process spawned [PID: {}]", ProcessId);
	crate::dev_log!("cocoon", "spawned PID={}", ProcessId);

	// Capture stdout for trace logging
	if let Some(stdout) = ChildProcess.stdout.take() {
		tokio::spawn(async move {
			let Reader = BufReader::new(stdout);
			let mut Lines = Reader.lines();

			while let Ok(Some(Line)) = Lines.next_line().await {
				dev_log!("cocoon", "[Cocoon stdout] {}", Line);
			}
		});
	}

	// Capture stderr for warn-level logging
	if let Some(stderr) = ChildProcess.stderr.take() {
		tokio::spawn(async move {
			let Reader = BufReader::new(stderr);
			let mut Lines = Reader.lines();

			while let Ok(Some(Line)) = Lines.next_line().await {
				dev_log!("cocoon", "warn: [Cocoon stderr] {}", Line);
			}
		});
	}

	// Establish Vine connection to Cocoon with retry loop
	let GRPCAddress = format!("127.0.0.1:{}", COCOON_GRPC_PORT);
	dev_log!(
		"cocoon",
		"[CocoonManagement] Connecting to Cocoon gRPC at {} (up to {} attempts, {}ms interval)...",
		GRPCAddress,
		GRPC_CONNECT_MAX_ATTEMPTS,
		GRPC_CONNECT_RETRY_INTERVAL_MS
	);

	let mut ConnectAttempt = 0u32;

	loop {
		ConnectAttempt += 1;
		crate::dev_log!(
			"grpc",
			"connecting to Cocoon at {} (attempt {}/{})",
			GRPCAddress,
			ConnectAttempt,
			GRPC_CONNECT_MAX_ATTEMPTS
		);

		match Vine::Client::ConnectToSideCar(SideCarIdentifier.clone(), GRPCAddress.clone()).await {
			Ok(()) => {
				crate::dev_log!("grpc", "connected to Cocoon on attempt {}", ConnectAttempt);
				break;
			},
			Err(Error) => {
				// Atom I12: Cocoon's gRPC server binds concurrently with
				// Mountain's first dial, so attempt 1 (and occasionally 2)
				// routinely hit `transport error` while Cocoon is still in
				// stage2_configuration. Log as a non-alarming probe retry
				// until the budget is exhausted; promote to "failed" only
				// on the final attempt so a pasted log makes a real
				// problem visually distinct from the expected startup
				// race.
				if ConnectAttempt >= GRPC_CONNECT_MAX_ATTEMPTS {
					crate::dev_log!(
						"grpc",
						"attempt {}/{} failed (final): {}",
						ConnectAttempt,
						GRPC_CONNECT_MAX_ATTEMPTS,
						Error
					);
					return Err(CommonError::IPCError {
						Description:format!(
							"Failed to connect to Cocoon gRPC at {} after {} attempts: {} (is Cocoon running?)",
							GRPCAddress, GRPC_CONNECT_MAX_ATTEMPTS, Error
						),
					});
				}

				crate::dev_log!(
					"grpc",
					"attempt {}/{} pending (Cocoon still booting): {}, retrying in {}ms",
					ConnectAttempt,
					GRPC_CONNECT_MAX_ATTEMPTS,
					Error,
					GRPC_CONNECT_RETRY_INTERVAL_MS
				);

				sleep(Duration::from_millis(GRPC_CONNECT_RETRY_INTERVAL_MS)).await;
			},
		}
	}

	dev_log!(
		"cocoon",
		"[CocoonManagement] Connected to Cocoon. Sending initialization data..."
	);

	// Brief delay to ensure Cocoon's gRPC service handlers are fully registered
	// after bindAsync resolves (race condition on fast connections like attempt 1)
	sleep(Duration::from_millis(200)).await;

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
			dev_log!(
				"cocoon",
				"[CocoonManagement] Cocoon handshake complete. Extension host is ready."
			);
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

	// Trigger startup extension activation. Cocoon is fully reactive -
	// it won't activate any extensions until Mountain tells it to.
	// Fire-and-forget: don't block on activation, and don't fail init if it errors.
	let SideCarId = SideCarIdentifier.clone();
	tokio::spawn(async move {
		// Small delay to let Cocoon finish processing the init response
		sleep(Duration::from_millis(500)).await;

		crate::dev_log!("exthost", "Sending $activateByEvent(\"*\") to Cocoon");

		if let Err(Error) = Vine::Client::SendRequest(
			&SideCarId,
			"$activateByEvent".to_string(),
			serde_json::json!({ "activationEvent": "*" }),
			30_000,
		)
		.await
		{
			dev_log!("cocoon", "warn: [CocoonManagement] $activateByEvent(\"*\") failed: {}", Error);
		} else {
			dev_log!("cocoon", "[CocoonManagement] Startup extensions activation triggered");
		}
	});

	// Store process handle for health monitoring and management
	{
		let mut state = COCOON_STATE.lock().await;
		state.ChildProcess = Some(ChildProcess);
		state.IsRunning = true;
		state.StartTime = Some(tokio::time::Instant::now());
		dev_log!("cocoon", "[CocoonManagement] Process state updated: Running");
	}

	// Reset health monitor on successful initialization
	{
		let mut health = COCOON_HEALTH.lock().await;
		health.clear_issues();
		dev_log!("cocoon", "[CocoonManagement] Health monitor reset to active state");
	}

	// Start background health monitoring
	let state_clone = Arc::clone(&COCOON_STATE);
	tokio::spawn(monitor_cocoon_health_task(state_clone));
	dev_log!("cocoon", "[CocoonManagement] Background health monitoring started");

	Ok(())
}

/// Background task that monitors Cocoon process health and logs crashes.
///
/// Once the child process has exited (or never existed), the monitor no
/// longer has anything useful to say - it exits quietly instead of
/// flooding the log with "No Cocoon process to monitor" every 5s, which
/// was rendering the dev log unreadable after any Cocoon crash.
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
					dev_log!(
						"cocoon",
						"warn: [CocoonHealth] Cocoon process crashed [PID: {}] [Exit Code: {}] [Uptime: {}s]",
						process_id.unwrap_or(0),
						exit_code_num,
						uptime
					);

					// Update state
					state_guard.IsRunning = false;
					state_guard.ChildProcess = None;
					COCOON_PID.store(0, std::sync::atomic::Ordering::Relaxed);

					// Report health issue
					{
						let mut health = COCOON_HEALTH.lock().await;
						health.add_issue(HealthIssue::Custom(format!("ProcessCrashed (Exit code: {})", exit_code_num)));
						dev_log!("cocoon", "warn: [CocoonHealth] Health score: {}", health.health_score);
					}

					// Log that automatic restart would be needed
					dev_log!(
						"cocoon",
						"warn: [CocoonHealth] CRASH DETECTED: Cocoon process has crashed and must be restarted \
						 manually or via application reinitialization"
					);
				},
				Ok(None) => {
					// Process is still running
					dev_log!(
						"cocoon",
						"[CocoonHealth] Cocoon process is healthy [PID: {}]",
						process_id.unwrap_or(0)
					);
				},
				Err(e) => {
					// Error checking process status
					dev_log!("cocoon", "warn: [CocoonHealth] Error checking process status: {}", e);

					// Report health issue
					{
						let mut health = COCOON_HEALTH.lock().await;
						health.add_issue(HealthIssue::Custom(format!("ProcessCheckError: {}", e)));
					}
				},
			}
		} else {
			// No child process exists - log exactly once, then exit the
			// monitor loop. Prior behaviour: flood the log with
			// "No Cocoon process to monitor" every 5s forever after a
			// crash, making the dev log unreadable. A future respawn will
			// spawn a fresh monitor via `StartCocoon`.
			dev_log!("cocoon", "[CocoonHealth] No Cocoon process to monitor - exiting monitor loop");
			drop(state_guard);
			return;
		}
	}
}

/// Atom I6: post-shutdown hard-kill. Called by RuntimeShutdown after the
/// `$shutdown` gRPC notification has been sent (and either succeeded or
/// timed out). Grabs the stored `Child` handle and force-terminates it if
/// still alive, then resets COCOON_STATE. This plugs the "Mountain exits
/// cleanly but child stays running" leak that leads to zombie-Cocoon
/// zombies holding the gRPC port.
///
/// Call AFTER the graceful $shutdown attempt - we don't want to race the
/// child's own cleanup. Safe to call with no stored child (no-op).
pub async fn HardKillCocoon() {
	let mut State = COCOON_STATE.lock().await;
	if let Some(mut Child) = State.ChildProcess.take() {
		let Pid = Child.id().unwrap_or(0);
		match Child.try_wait() {
			Ok(Some(_Status)) => {
				dev_log!("cocoon", "[CocoonShutdown] Child PID {} already exited; clearing handle.", Pid);
			},
			Ok(None) => {
				dev_log!(
					"cocoon",
					"[CocoonShutdown] Child PID {} still alive after $shutdown; sending SIGKILL.",
					Pid
				);
				if let Err(Error) = Child.start_kill() {
					dev_log!("cocoon", "warn: [CocoonShutdown] start_kill failed on PID {}: {}", Pid, Error);
				}
				// Best-effort wait so the OS reaps and frees the port.
				let _ = tokio::time::timeout(std::time::Duration::from_secs(2), Child.wait()).await;
			},
			Err(Error) => {
				dev_log!("cocoon", "warn: [CocoonShutdown] try_wait failed on PID {}: {}", Pid, Error);
			},
		}
	}
	State.IsRunning = false;
}

/// Atom I6: pre-boot sweep. TCP-probe the Cocoon gRPC port and kill any
/// stale process still bound to it. Prevents the EADDRINUSE cascade that
/// leaves the extension host in degraded mode when a prior Mountain exited
/// without cleaning up its child.
///
/// Behaviour:
/// - If the port answers a TCP connect, assume an owner is listening.
/// - Use `lsof -nP -iTCP:<port> -sTCP:LISTEN -t` (macOS/Linux) to resolve the
///   PID. `lsof` is ubiquitous on macOS/Linux and doesn't require root for
///   local user-owned processes.
/// - SIGTERM first, 500ms grace window, then SIGKILL if still alive.
/// - Logs every step via `dev_log!("cocoon", …)` so the sweep is visible in
///   Mountain.dev.log without parsing stderr.
/// - Best-effort: failures don't abort Mountain boot. A real EADDRINUSE later
///   will surface via Cocoon's own bootstrap error.
fn SweepStaleCocoon(Port:u16) {
	use std::{net::TcpStream, time::Duration};

	let Addr = format!("127.0.0.1:{}", Port);

	// Cheap liveness probe. Timeout is aggressive - zombie ports answer
	// immediately; a clean port is ECONNREFUSED and returns instantly.
	let Probe =
		TcpStream::connect_timeout(&Addr.parse().expect("valid socket addr literal"), Duration::from_millis(200));
	if Probe.is_err() {
		dev_log!("cocoon", "[CocoonSweep] Port {} is clean (no prior listener).", Port);
		return;
	}

	dev_log!(
		"cocoon",
		"[CocoonSweep] Port {} has a listener - attempting to resolve owner via lsof.",
		Port
	);

	// `lsof -nP -iTCP:<port> -sTCP:LISTEN -t` → one PID per line.
	let LsofOutput = std::process::Command::new("lsof")
		.args(["-nP", &format!("-iTCP:{}", Port), "-sTCP:LISTEN", "-t"])
		.output();

	let Output = match LsofOutput {
		Ok(O) => O,
		Err(Error) => {
			dev_log!(
				"cocoon",
				"warn: [CocoonSweep] lsof unavailable ({}). Skipping sweep; Cocoon spawn may fail with EADDRINUSE.",
				Error
			);
			return;
		},
	};

	if !Output.status.success() {
		dev_log!("cocoon", "warn: [CocoonSweep] lsof exited non-zero. Skipping sweep.");
		return;
	}

	let Stdout = String::from_utf8_lossy(&Output.stdout);
	let Pids:Vec<i32> = Stdout.lines().filter_map(|L| L.trim().parse::<i32>().ok()).collect();

	if Pids.is_empty() {
		dev_log!(
			"cocoon",
			"warn: [CocoonSweep] Port {} answered but lsof found no LISTEN PID - giving up.",
			Port
		);
		return;
	}

	// Guard against self-kill. Mountain currently binds 50051, not Cocoon's
	// 50052, but belt-and-braces for future refactors.
	let SelfPid = std::process::id() as i32;
	for Pid in Pids {
		if Pid == SelfPid {
			dev_log!(
				"cocoon",
				"warn: [CocoonSweep] Port {} owned by Mountain itself (PID {}); refusing to kill.",
				Port,
				Pid
			);
			continue;
		}
		dev_log!("cocoon", "[CocoonSweep] Killing stale PID {} (SIGTERM).", Pid);
		let _ = std::process::Command::new("kill").arg(Pid.to_string()).status();
		std::thread::sleep(Duration::from_millis(500));
		// Recheck - if still alive, escalate.
		let StillAlive = std::process::Command::new("kill")
			.args(["-0", &Pid.to_string()])
			.status()
			.map(|S| S.success())
			.unwrap_or(false);
		if StillAlive {
			dev_log!("cocoon", "warn: [CocoonSweep] PID {} survived SIGTERM; sending SIGKILL.", Pid);
			let _ = std::process::Command::new("kill").args(["-9", &Pid.to_string()]).status();
			std::thread::sleep(Duration::from_millis(200));
		}
		dev_log!("cocoon", "[CocoonSweep] PID {} reaped.", Pid);
	}
}
