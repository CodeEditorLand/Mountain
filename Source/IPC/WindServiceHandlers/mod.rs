//! Wind Service Handlers - dispatcher and sub-module aggregator.
//! Domain files handle the individual handler implementations.

pub mod Cocoon;

#[path = "Commands/mod.rs"]
pub mod Commands;

#[path = "Configuration/mod.rs"]
pub mod Configuration;

pub mod Encryption;

pub mod Extension;

pub mod ExtensionHost;

pub mod Extensions;

pub mod FileSystem;

pub mod Git;

pub mod History;

pub mod Model;

pub mod NativeDialog;

pub mod NativeHost;

pub mod Navigation;

pub mod Output;

#[path = "Search/mod.rs"]
pub mod Search;

pub mod Sky;

pub mod Storage;

pub mod Terminal;

pub mod UI;

pub mod TreeView;

pub mod Update;

pub mod Workspaces;

pub mod DispatchMatch;

pub mod Dispatcher;

pub mod Utilities;

// ============================================================================
// Thin forwarding wrappers - all dispatch logic lives in DispatchMatch.rs.
// ============================================================================

/// Forward to the main IPC dispatch function in DispatchMatch.rs.
pub async fn mountain_ipc_invoke(
	ApplicationHandle:tauri::AppHandle,

	command:String,

	Arguments:Vec<serde_json::Value>,
) -> Result<serde_json::Value, String> {
	DispatchMatch::mountain_ipc_invoke(ApplicationHandle, command, Arguments).await
}

/// Forward to handler registration in DispatchMatch.rs.
pub fn register_wind_ipc_handlers(ApplicationHandle:&tauri::AppHandle) -> Result<(), String> {
	DispatchMatch::register_wind_ipc_handlers(ApplicationHandle)
}
