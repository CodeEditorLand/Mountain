// @module CocoonManagement
// @description Contains the logic for launching, managing the lifecycle of,
// and performing the initial handshake with the Cocoon sidecar process.

use std::{collections::HashMap, env, process::Stdio, time::Duration};

use Common::error::CommonError;
use log::{error, info, trace, warn};
use tauri::{AppHandle, Manager, Runtime};
use tokio::{
	io::{AsyncBufReadExt, BufReader},
	process::Command,
	time::sleep,
};

use super::InitializationData;
use crate::{ApplicationState::ApplicationState::ApplicationState, Vine};

/// The main entry point for starting the Cocoon process manager.
/// It spawns a background task that handles the entire lifecycle.
pub async fn InitializeCocoon<R:Runtime>(app_handle:&AppHandle<R>) {
	info!("[ProcessManagement] Initializing Cocoon sidecar manager...");
	#[cfg(feature = "extension_host_cocoon")]
	{
		let app_handle_clone = app_handle.clone();
		tokio::spawn(async move {
			if let Err(e) = launch_and_manage_cocoon_sidecar(app_handle_clone).await {
				error!("[ProcessManagement] CRITICAL: Failed to launch and manage Cocoon: {}", e);
				// TODO: In a real app, this should notify the user that
				// extensions will not work.
			}
		});
	}
	#[cfg(not(feature = "extension_host_cocoon"))]
	{
		info!("[ProcessManagement] 'extension_host_cocoon' feature is disabled. Cocoon will not launch.");
	}
}

/// Spawns the Cocoon process and manages its communication and handshake.
async fn launch_and_manage_cocoon_sidecar<R:Runtime>(app_handle:AppHandle<R>) -> Result<(), CommonError> {
	let sidecar_id = "cocoon-main".to_string();
	let path_resolver = app_handle.path_resolver();

	let script_path = match path_resolver.resolve_resource("scripts/cocoon/bootstrap-fork.js") {
		Some(path) if path.exists() => path,
		_ => return Err(CommonError::FsNotFound("Cocoon bootstrap-fork.js script not found.".into())),
	};

	let mut node_command = Command::new("node");

	// --- Construct Environment Variables ---
	let mut Environment_variables = HashMap::new();
	Environment_variables.insert("VSCODE_PIPE_LOGGING".to_string(), "true".to_string());
	Environment_variables.insert("VSCODE_VERBOSE_LOGGING".to_string(), "true".to_string());
	Environment_variables.insert("VSCODE_PARENT_PID".to_string(), std::process::id().to_string());
	Environment_variables.insert("VSCODE_HANDLES_UNCAUGHT_ERRORS".to_string(), "true".to_string());
	// These ports should be configurable and dynamically allocated.
	Environment_variables.insert("MOUNTAIN_GRPC_PORT".to_string(), "50051".to_string());
	Environment_variables.insert("COCOON_GRPC_PORT".to_string(), "50052".to_string());

	// --- Setup Command ---
	node_command
		.arg(&script_path)
		.env_clear()
		.envs(Environment_variables)
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped());

	// --- Spawn Process ---
	let mut child_process = node_command
		.spawn()
		.map_err(|e| CommonError::IpcError { Description:format!("Failed to spawn Cocoon: {}", e) })?;
	info!("[ProcessManagement] Cocoon process spawned [PID: {:?}]", child_process.id());

	// Spawn tasks to log stdout and stderr from the sidecar.
	if let Some(stdout) = child_process.stdout.take() {
		tokio::spawn(async move {
			let reader = BufReader::new(stdout);
			let mut lines = reader.lines();
			while let Some(line) = lines.next_line().await.unwrap_or(None) {
				trace!("[Cocoon stdout] {}", line);
			}
		});
	}
	if let Some(stderr) = child_process.stderr.take() {
		tokio::spawn(async move {
			let reader = BufReader::new(stderr);
			let mut lines = reader.lines();
			while let Some(line) = lines.next_line().await.unwrap_or(None) {
				warn!("[Cocoon stderr] {}", line);
			}
		});
	}

	// --- Perform Handshake ---
	// Wait for Cocoon to start its gRPC server and then connect to it.
	info!("[ProcessManagement] Waiting for Cocoon gRPC server to start...");
	sleep(Duration::from_millis(2000)).await; // Simple delay. A robust solution would use a ready signal.
	Vine::client::ConnectToSidecar(sidecar_id.clone(), "127.0.0.1:50052".to_string()).await?;

	info!("[ProcessManagement] Cocoon is ready. Sending initialization data...");
	let app_state = app_handle.state::<ApplicationState>();
	let main_init_data = InitializationData::ConstructExtensionHostInitializationData(&app_handle, &app_state);

	let response =
		Vine::client::SendRequest(&sidecar_id, "initExtensionHost".to_string(), main_init_data, 60000).await?;

	let response_string = response.as_str().unwrap_or("");
	if response_string == "initialized" {
		info!("[ProcessManagement] Cocoon handshake complete.");
	} else {
		return Err(CommonError::IpcError {
			Description:format!("Cocoon initialization failed with response: {}", response_string),
		});
	}

	Ok(())
}
