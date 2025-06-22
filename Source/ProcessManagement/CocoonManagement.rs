// File: Mountain/Source/ProcessManagement/CocoonManagement.rs
// Role: Contains the logic for launching, managing the lifecycle of, and
// performing the initial handshake with the Cocoon sidecar process.

//! # CocoonManagement
//!
//! Contains the logic for launching, managing the lifecycle of, and performing
//! the initial handshake with the Cocoon sidecar process.

use std::{collections::HashMap, process::Stdio, sync::Arc, time::Duration};

use Common::Error::CommonError::CommonError;
use log::{error, info, trace, warn};
use tauri::{
	AppHandle,
	Manager,
	Wry,
	path::{BaseDirectory, PathResolver},
};
use tokio::{
	io::{AsyncBufReadExt, BufReader},
	process::Command,
	time::sleep,
};

use super::InitializationData;
use crate::{Environment::MountainEnvironment::MountainEnvironment, Vine};

/// The main entry point for starting the Cocoon process manager.
pub async fn InitializeCocoon(ApplicationHandle:&AppHandle, Environment:&Arc<MountainEnvironment>) {
	info!("[CocoonManagement] Initializing Cocoon sidecar manager...");

	#[cfg(feature = "ExtensionHostCocoon")]
	{
		// Clone the handles for the spawned task.
		let ApplicationHandleClone = ApplicationHandle.clone();

		let EnvironmentClone = Environment.clone();

		tokio::spawn(async move {
			if let Err(e) = LaunchAndManageCocoonSidecar(ApplicationHandleClone, EnvironmentClone).await {
				error!("[CocoonManagement] CRITICAL: Failed to launch and manage Cocoon: {}", e);
			}
		});
	}

	#[cfg(not(feature = "ExtensionHostCocoon"))]
	{
		info!("[CocoonManagement] 'ExtensionHostCocoon' feature is disabled. Cocoon will not be launched.");
	}
}

/// Spawns the Cocoon process and manages its communication and handshake.
async fn LaunchAndManageCocoonSidecar(
	ApplicationHandle:AppHandle,

	Environment:Arc<MountainEnvironment>,
) -> Result<(), CommonError> {
	let SidecarIdentifier = "cocoon-main".to_string();

	let path_resolver:PathResolver<Wry> = ApplicationHandle.path().clone();

	let ScriptPath = path_resolver
		.resolve("scripts/cocoon/bootstrap-fork.js", BaseDirectory::Resource)
		.map_err(|e| CommonError::FileSystemNotFound(e.to_string().into()))?;

	if !ScriptPath.exists() {
		return Err(CommonError::FileSystemNotFound(
			"Cocoon bootstrap-fork.js script not found.".into(),
		));
	}

	let mut NodeCommand = Command::new("node");

	let mut EnvironmentVariables = HashMap::new();

	EnvironmentVariables.insert("VSCODE_PIPE_LOGGING".to_string(), "true".to_string());

	EnvironmentVariables.insert("VSCODE_VERBOSE_LOGGING".to_string(), "true".to_string());

	EnvironmentVariables.insert("VSCODE_PARENT_PID".to_string(), std::process::id().to_string());

	EnvironmentVariables.insert("MOUNTAIN_GRPC_PORT".to_string(), "50051".to_string());

	EnvironmentVariables.insert("COCOON_GRPC_PORT".to_string(), "50052".to_string());

	NodeCommand
		.arg(&ScriptPath)
		.env_clear()
		.envs(EnvironmentVariables)
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped());

	let mut ChildProcess = NodeCommand
		.spawn()
		.map_err(|e| CommonError::IPCError { Description:format!("Failed to spawn Cocoon: {}", e) })?;

	info!("[CocoonManagement] Cocoon process spawned [PID: {:?}]", ChildProcess.id());

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

	info!("[CocoonManagement] Waiting for Cocoon gRPC server to start...");

	sleep(Duration::from_millis(2000)).await;

	// Assuming the sidecar listens on a standard gRPC port.
	Vine::Client::ConnectToSidecar(SidecarIdentifier.clone(), "127.0.0.1:50052".to_string())
		.await
		.map_err(|e| CommonError::IPCError { Description:e.to_string() })?;

	info!("[CocoonManagement] Cocoon is ready. Sending initialization data...");

	let MainInitializationData = InitializationData::ConstructExtensionHostInitializationData(&Environment).await?;

	let Response = Vine::Client::SendRequest(
		&SidecarIdentifier,
		"InitializeExtensionHost".to_string(),
		MainInitializationData,
		60000,
	)
	.await
	.map_err(|e| CommonError::IPCError { Description:e.to_string() })?;

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
