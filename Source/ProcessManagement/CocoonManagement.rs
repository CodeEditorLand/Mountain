// File: Mountain/Source/ProcessManagement/CocoonManagement.rs
// Role: Contains the logic for launching, managing the lifecycle of, and
// performing the initial handshake with the Cocoon sidecar process.

//! # CocoonManagement
//!
//! Contains the logic for launching, managing the lifecycle of, and performing
//! the initial handshake with the Cocoon sidecar process.

#![allow(non_snake_case, non_camel_case_types)]

use std::{collections::HashMap, process::Stdio, sync::Arc, time::Duration};

use Common::Error::CommonError::CommonError;
use log::{info, trace, warn};
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

/// The main entry point for starting the Cocoon process manager. This function
/// now returns a Result to indicate if initialization was successful.
pub async fn InitializeCocoon(
	ApplicationHandle:&AppHandle,

	Environment:&Arc<MountainEnvironment>,
) -> Result<(), CommonError> {
	info!("[CocoonManagement] Initializing Cocoon sidecar manager...");

	#[cfg(feature = "ExtensionHostCocoon")]
	{
		// Awaiting this directly now, so the caller knows if it failed.
		LaunchAndManageCocoonSideCar(ApplicationHandle.clone(), Environment.clone()).await
	}

	#[cfg(not(feature = "ExtensionHostCocoon"))]
	{
		info!("[CocoonManagement] 'ExtensionHostCocoon' feature is disabled. Cocoon will not be launched.");

		Ok(())
	}
}

/// Spawns the Cocoon process and manages its communication and handshake.
async fn LaunchAndManageCocoonSideCar(
	ApplicationHandle:AppHandle,

	Environment:Arc<MountainEnvironment>,
) -> Result<(), CommonError> {
	let SideCarIdentifier = "cocoon-main".to_string();

	let path_resolver:PathResolver<Wry> = ApplicationHandle.path().clone();

	let ScriptPath = path_resolver
		.resolve("scripts/cocoon/bootstrap-fork.js", BaseDirectory::Resource)
		.map_err(|Error| CommonError::FileSystemNotFound(Error.to_string().into()))?;

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
		.map_err(|Error| CommonError::IPCError { Description:format!("Failed to spawn Cocoon: {}", Error) })?;

	info!("[CocoonManagement] Cocoon process spawned [PID: {:?}]", ChildProcess.id());

	if let Some(stdout) = ChildProcess.stdout.take() {
		tokio::spawn(async move {
			let Reader = BufReader::new(stdout);

			let mut Lines = Reader.lines();

			while let Ok(Some(Line)) = Lines.next_line().await {
				trace!("[Cocoon stdout] {}", Line);
			}
		});
	}
	if let Some(stderr) = ChildProcess.stderr.take() {
		tokio::spawn(async move {
			let Reader = BufReader::new(stderr);

			let mut Lines = Reader.lines();

			while let Ok(Some(Line)) = Lines.next_line().await {
				warn!("[Cocoon stderr] {}", Line);
			}
		});
	}

	info!("[CocoonManagement] Waiting for Cocoon gRPC server to start...");

	sleep(Duration::from_millis(2000)).await;

	Vine::Client::ConnectToSideCar(SideCarIdentifier.clone(), "127.0.0.1:50052".to_string())
		.await
		.map_err(|Error| CommonError::IPCError { Description:Error.to_string() })?;

	info!("[CocoonManagement] Cocoon is ready. Sending initialization data...");

	let MainInitializationData = InitializationData::ConstructExtensionHostInitializationData(&Environment).await?;

	let Response = Vine::Client::SendRequest(
		&SideCarIdentifier,
		"InitializeExtensionHost".to_string(),
		MainInitializationData,
		60000,
	)
	.await
	.map_err(|Error| CommonError::IPCError { Description:Error.to_string() })?;

	if Response.as_str() == Some("initialized") {
		info!("[CocoonManagement] Cocoon handshake complete.");
	} else {
		return Err(CommonError::IPCError {
			Description:format!("Cocoon initialization failed with response: {}", Response),
		});
	}

	Ok(())
}
