//! # IPC Server Register Module
//!
//! Initializes and manages the IPC server in Tauri state.

use crate::IPC::TauriIPCServer::TauriIPCServer;
use log::info;

/// Creates and initializes the Tauri IPC server, storing it in Tauri state.
///
/// # Arguments
///
/// * `ApplicationHandle` - The Tauri application handle
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # IPC Server Functionality
///
/// The Tauri IPC server provides:
/// - Message routing between Wind and native code
/// - Status reporting for IPC operations
/// - Command invocation for frontend communication
/// - Configuration synchronization
///
/// # Errors
///
/// Returns an error if IPC server initialization or state management fails.
pub fn IPCServerRegister(
	ApplicationHandle: &tauri::AppHandle,
) -> Result<TauriIPCServer, String> {
	let ipc_server = TauriIPCServer::new(ApplicationHandle.clone());

	ApplicationHandle.manage(ipc_server.clone());

	info!("[IPC] [Server] IPC server initialized and managed.");

	Ok(ipc_server)
}
