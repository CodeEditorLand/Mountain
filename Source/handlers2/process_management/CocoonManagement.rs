use std::{collections::HashMap, env, process::Stdio, sync::Arc};

use Common::error::CommonError;
use log::{error, info, trace, warn};
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};
use tokio::{
	io::{AsyncBufReadExt, BufReader},
	process::Command,
	sync::oneshot,
};

/// @module CocoonManagement
/// @description Contains the logic for launching, managing the lifecycle of,
/// and performing the initial handshake with the Cocoon sidecar process.
use crate::{
	AppState::AppState::AppState,
	handlers::process_management::InitData,
	vine::{self, client::GetSidecarClient},
};

/// The main entry point for starting the Cocoon process manager.
/// It spawns a background task that handles the entire lifecycle.
pub async fn InitializeCocoon<R:Runtime>(AppHandle:&AppHandle<R>) {
	info!("[ProcessManagement] Initializing Cocoon sidecar manager...");
	#[cfg(feature = "extension_host_cocoon")]
	{
		let AppHandleClone = AppHandle.clone();
		tokio::spawn(async move {
			if let Err(e) = LaunchAndManageCocoonSidecar(AppHandleClone).await {
				error!("[ProcessManagement] Failed to launch and manage Cocoon: {}", e);
			}
		});
	}
	#[cfg(not(feature = "extension_host_cocoon"))]
	{
		info!("[ProcessManagement] 'extension_host_cocoon' feature is disabled. Cocoon will not launch.");
	}
}

/// Spawns the Cocoon process and manages its communication and handshake.
async fn LaunchAndManageCocoonSidecar<R:Runtime>(AppHandle:AppHandle<R>) -> Result<(), CommonError> {
	let SidecarId = "cocoon-main".to_string();
	let PathResolver = AppHandle.path_resolver();

	let ScriptPath = match PathResolver.resolve_resource("scripts/cocoon/bootstrap-fork.js") {
		Some(path) if path.exists() => path,
		_ => return Err(CommonError::FsNotFound("Cocoon bootstrap-fork.js script not found.".into())),
	};

	let mut NodeCommand = Command::new("node");

	// --- Construct Environment Variables ---
	let mut EnvironmentVariables = HashMap::new();
	EnvironmentVariables.insert("VSCODE_PIPE_LOGGING".to_string(), "true".to_string());
	EnvironmentVariables.insert("VSCODE_VERBOSE_LOGGING".to_string(), "true".to_string());
	EnvironmentVariables.insert("VSCODE_PARENT_PID".to_string(), std::process::id().to_string());
	EnvironmentVariables.insert("VSCODE_HANDLES_UNCAUGHT_ERRORS".to_string(), "true".to_string());
	EnvironmentVariables.insert("MOUNTAIN_GRPC_PORT".to_string(), "50051".to_string()); // Configurable
	EnvironmentVariables.insert("COCOON_GRPC_PORT".to_string(), "50052".to_string()); // Configurable

	// --- Setup Command ---
	NodeCommand
		.arg(ScriptPath)
		.env_clear()
		.envs(EnvironmentVariables)
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped());

	// --- Spawn Process ---
	let mut ChildProcess = NodeCommand
		.spawn()
		.map_err(|e| CommonError::IpcError { Description:format!("Failed to spawn Cocoon: {}", e) })?;
	info!("[ProcessManagement] Cocoon process spawned [PID: {:?}]", ChildProcess.id());

	// Spawn tasks to log stdout and stderr from the sidecar.
	let stdout = ChildProcess.stdout.take().expect("Failed to capture stdout");
	let stderr = ChildProcess.stderr.take().expect("Failed to capture stderr");
	tokio::spawn(async move {
		for line in BufReader::new(stdout).lines().next_line().await {
			trace!("[Cocoon stdout] {}", line.unwrap_or_default());
		}
	});
	tokio::spawn(async move {
		for line in BufReader::new(stderr).lines().next_line().await {
			warn!("[Cocoon stderr] {}", line.unwrap_or_default());
		}
	});

	// --- Perform Handshake ---
	let (tx, rx) = oneshot::channel::<()>();
	// TODO: A real implementation would use the `vine::client` to listen for the
	// `$initialHandshake` notification. For now, we simulate this with a delay and
	// a oneshot channel.
	tokio::spawn(async move {
		tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await; // Wait for Cocoon to start its gRPC server
		tx.send(()).ok();
	});

	info!("[ProcessManagement] Waiting for Cocoon 'Ready' signal...");
	rx.await.map_err(|_| {
		CommonError::IpcError { Description:"Handshake with Cocoon failed; signal channel closed.".to_string() }
	})?;

	info!("[ProcessManagement] Cocoon is ready. Sending initialization data...");
	let AppStateInstance = AppHandle.state::<AppState>();
	let MainInitData = InitData::ConstructExtensionHostInitData(&AppHandle, &AppStateInstance);

	let Response = vine::client::SendRequest(&SidecarId, "initExtensionHost".to_string(), MainInitData, 60000).await?;

	let ResponseString = Response.as_str().unwrap_or("");
	if ResponseString == "initialized" {
		info!("[ProcessManagement] Cocoon handshake complete.");
	} else {
		return Err(CommonError::IpcError {
			Description:format!("Cocoon initialization failed with response: {}", ResponseString),
		});
	}

	Ok(())
}
