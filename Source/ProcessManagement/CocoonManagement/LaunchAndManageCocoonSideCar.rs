//! Spawns the Cocoon process, manages its communication channels, and
//! performs the complete initialization handshake sequence, then starts
//! health monitoring and the automatic-restart handler.

use std::{process::Stdio, sync::Arc};

use CommonLibrary::Error::CommonError::CommonError;
use tauri::AppHandle;
use tokio::process::Command;

use crate::{Environment::MountainEnvironment::MountainEnvironment, ProcessManagement::NodeResolver, dev_log};

/// This function implements the complete Cocoon lifecycle:
/// 1. Validates bootstrap script availability
/// 2. Constructs environment variables for IPC and logging
/// 3. Spawns Node.js process with proper IO redirection
/// 4. Captures stdout/stderr for logging
/// 5. Waits for gRPC server to be ready
/// 6. Establishes Vine connection
/// 7. Sends initialization payload and validates response
/// # Arguments
/// * `ApplicationHandle` - Tauri application handle for resolving resource
///   paths
/// * `Environment` - Mountain environment containing application state
/// # Returns
/// * `Ok(())` - Cocoon process spawned, connected, and initialized successfully
/// * `Err(CommonError)` - Any failure during the initialization sequence
/// # Errors
/// - `FileSystemNotFound`: Bootstrap script not found in resources
/// - `IPCError`: Failed to spawn process, connect gRPC, or complete handshake
/// # Lifecycle
/// The process runs as a background task with IO redirection for logging.
/// Process failures are logged but not automatically restarted (callers should
/// implement restart strategies based on their requirements).
pub(crate) async fn Fn(ApplicationHandle:AppHandle, Environment:Arc<MountainEnvironment>) -> Result<(), CommonError> {
	let SideCarIdentifier = super::COCOON_SIDE_CAR_IDENTIFIER.to_string();

	let ScriptPath = super::ResolveBootstrapScript::Fn(&ApplicationHandle)?;

	// Atom I6: zombie-Cocoon sweep. If a prior Mountain exited without
	// killing its child (segfault, SIGKILL, debugger detach, …), the stale
	// node process keeps port COCOON_GRPC_PORT bound. The new Mountain's
	// VineClient then "successfully connects" to the zombie while the
	// freshly-spawned Cocoon fails to bind with EADDRINUSE, and the whole
	// extension host enters degraded mode with zero extensions visible.
	//
	// Probe the port. If it answers, find the owning PID via `lsof -t -i
	// :<port>` and SIGTERM → 500ms wait → SIGKILL. Then proceed as normal.
	super::SweepStaleCocoon::Fn(super::COCOON_GRPC_PORT);

	// Atom N1: resolve Node binary via NodeResolver (shipped → version
	// managers → homebrew → PATH). Logs the pick + source for forensics.
	// Overridable via `Pick=/absolute/path/to/node`.
	let ResolvedNodeBinary = NodeResolver::ResolveNodeBinary::Fn(&ApplicationHandle);

	let mut NodeCommand = Command::new(&ResolvedNodeBinary.Path);

	NodeCommand
		.arg(&ScriptPath)
		.env_clear()
		.envs(super::BuildCocoonEnvironment::Fn())
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped());

	// Spawn the process with error handling
	let mut ChildProcess = NodeCommand.spawn().map_err(|Error| {
		CommonError::IPCError {
			Description:format!(
				"Failed to spawn Cocoon with node={} (source={}): {}. Override with Pick=/absolute/path or install \
				 Node.js.",
				ResolvedNodeBinary.Path.display(),
				ResolvedNodeBinary.Source.AsLabel(),
				Error
			),
		}
	})?;

	let ProcessId = ChildProcess.id().unwrap_or(0);

	super::COCOON_PID.store(ProcessId, std::sync::atomic::Ordering::Relaxed);

	dev_log!("cocoon", "[CocoonManagement] Cocoon process spawned [PID: {}]", ProcessId);

	crate::dev_log!("cocoon", "spawned PID={}", ProcessId);

	super::SpawnCocoonIoForwarders::Fn(&mut ChildProcess);

	super::ConnectToCocoonGrpc::Fn(&SideCarIdentifier, &mut ChildProcess).await?;

	super::SendInitializationHandshake::Fn(&SideCarIdentifier, &Environment).await?;

	super::SpawnStartupActivationTask::Fn(&SideCarIdentifier, &Environment);

	// Store process handle for health monitoring and management
	{
		let mut state = super::COCOON_STATE.lock().await;

		state.ChildProcess = Some(ChildProcess);

		state.IsRunning = true;

		state.StartTime = Some(tokio::time::Instant::now());

		dev_log!("cocoon", "[CocoonManagement] Process state updated: Running");
	}

	// Reset health monitor on successful initialization
	{
		let mut health = super::COCOON_HEALTH.lock().await;

		health.ClearIssues();

		dev_log!("cocoon", "[CocoonManagement] Health monitor reset to active state");
	}

	// Wire up the automatic-restart channel. The health monitor sends a
	// backoff duration (in seconds) on crash; the handler task sleeps then
	// calls LaunchAndManageCocoonSideCar to respawn Cocoon.
	super::SpawnRestartHandler::Fn(&ApplicationHandle, &Environment).await;

	// Start background health monitoring
	let state_clone = Arc::clone(&super::COCOON_STATE);

	tokio::spawn(super::MonitorCocoonHealthTask::Fn(state_clone));

	dev_log!("cocoon", "[CocoonManagement] Background health monitoring started");

	Ok(())
}
