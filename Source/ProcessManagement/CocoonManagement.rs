//! # CocoonManagement
//!
//! Contains the logic for launching, managing the lifecycle of, and performing
//! the initial handshake with the Cocoon sidecar process.

use std::{collections::HashMap, process::Stdio, time::Duration};

use Common::Error::CommonError::CommonError;
use log::{error, info, trace, warn};
use tauri::{AppHandle, Manager};
use tokio::{
	io::{AsyncBufReadExt, BufReader},
	process::Command,
	time::sleep,
};

use super::InitializationData;
use crate::{ApplicationState::ApplicationState::ApplicationState, Vine};

/// The main entry point for starting the Cocoon process manager.
/// It spawns a background task that handles the entire lifecycle.
pub async fn InitializeCocoon(ApplicationHandle:&AppHandle) {
	info!("[CocoonManagement] Initializing Cocoon sidecar manager...");
	#[cfg(feature = "extension_host_cocoon")]
	{
		let ApplicationHandleClone = ApplicationHandle.clone();
		tokio::spawn(async move {
			if let Err(e) = LaunchAndManageCocoonSidecar(ApplicationHandleClone).await {
				error!("[CocoonManagement] CRITICAL: Failed to launch and manage Cocoon: {}", e);
				// In a real app, this should notify the user that extensions
				// will not work.
			}
		});
	}
	#[cfg(not(feature = "extension_host_cocoon"))]
	{
		info!("[CocoonManagement] 'extension_host_cocoon' feature is disabled. Cocoon will not be launched.");
	}
}

/// Spawns the Cocoon process and manages its communication and handshake.
async fn LaunchAndManageCocoonSidecar(ApplicationHandle:AppHandle) -> Result<(), CommonError> {
	let SidecarIdentifier = "cocoon-main".to_string();
	let PathResolver = ApplicationHandle.path_resolver();

	let ScriptPath = match PathResolver.resolve_resource("scripts/cocoon/bootstrap-fork.js") {
		Some(path) if path.exists() => path,
		_ => {
			return Err(CommonError::FileSystemNotFound(
				"Cocoon bootstrap-fork.js script not found.".into(),
			));
		},
	};

	let mut NodeCommand = Command::new("node");

	// --- Construct Environment Variables ---
	let mut EnvironmentVariables = HashMap::new();
	EnvironmentVariables.insert("VSCODE_PIPE_LOGGING".to_string(), "true".to_string());
	EnvironmentVariables.insert("VSCODE_VERBOSE_LOGGING".to_string(), "true".to_string());
	EnvironmentVariables.insert("VSCODE_PARENT_PID".to_string(), std::process::id().to_string());
	// These ports should be configurable and dynamically allocated in a real app.
	EnvironmentVariables.insert("MOUNTAIN_GRPC_PORT".to_string(), "50051".to_string());
	EnvironmentVariables.insert("COCOON_GRPC_PORT".to_string(), "50052".to_string());

	// --- Setup Command ---
	NodeCommand
		.arg(&ScriptPath)
		.env_clear()
		.envs(EnvironmentVariables)
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped());

	// --- Spawn Process ---
	let mut ChildProcess = NodeCommand
		.spawn()
		.map_err(|e| CommonError::IPCError { Description:format!("Failed to spawn Cocoon: {}", e) })?;
	info!("[CocoonManagement] Cocoon process spawned [PID: {:?}]", ChildProcess.id());

	// Spawn tasks to log stdout and stderr from the sidecar.
	if let Some(stdout) = ChildProcess.stdout.take() {
		tokio::spawn(async move {
			let Reader = BufReader::new(stdout);
			let mut Lines = Reader.lines();
			while let Some(Line) = Lines.next_line().await.unwrap_or(None) {
				trace!("[Cocoon stdout] {}", Line);
			}
		});
	}
	if let Some(stderr) = ChildProcess.stderr.take() {
		tokio::spawn(async move {
			let Reader = BufReader::new(stderr);
			let mut Lines = Reader.lines();
			while let Some(Line) = Lines.next_line().await.unwrap_or(None) {
				warn!("[Cocoon stderr] {}", Line);
			}
		});
	}

	// --- Perform Handshake ---
	info!("[CocoonManagement] Waiting for Cocoon gRPC server to start...");
	// A robust solution would use a ready signal (e.g., a specific log line).
	sleep(Duration::from_millis(2000)).await;
	Vine::Client::ConnectToSidecar(SidecarIdentifier.clone(), "127.0.0.1:50052".to_string()).await?;

	info!("[CocoonManagement] Cocoon is ready. Sending initialization data...");
	let AppState = ApplicationHandle.try_state::<ApplicationState>()?;
	let MainInitializationData =
		InitializationData::ConstructExtensionHostInitializationData(&ApplicationHandle, &AppState);

	let Response = Vine::Client::SendRequest(
		&SidecarIdentifier,
		"InitializeExtensionHost".to_string(),
		MainInitializationData,
		60000,
	)
	.await?;

	let ResponseString = Response.as_str().unwrap_or("");
	if ResponseString == "initialized" {
		info!("[CocoonManagement] Cocoon handshake complete.");
	} else {
		return Err(CommonError::IPCError {
			Description:format!("Cocoon initialization failed with response: {}", ResponseString),
		});
	}

	Ok(())
}
