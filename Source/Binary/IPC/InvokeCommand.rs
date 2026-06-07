//! # InvokeCommand
//!
//! Invokes IPC methods for Wind service communication.
//!
//! ## RESPONSIBILITIES
//!
//! ### Method Invocation
//! - Accept method invocation requests from frontend
//! - Delegate to WindServiceHandlers for processing
//! - Return method execution results
//! - Handle method parameters validation
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC wrapper command in Binary subsystem
//! - Bridge to Wind service handlers
//!
//! ### Dependencies
//! - crate::IPC::WindServiceHandlers: Method execution
//! - tauri: IPC framework
//! - serde_json: JSON serialization
//!
//! ### Dependents
//! - Wind frontend: Invokes methods via this command
//! - Tauri IPC handler: Routes invocation requests
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Validate method names to prevent unauthorized access
//! - Sanitize method parameters before execution
//! - Restrict access to privileged methods
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Method execution varies by implementation
//! - Consider async for long-running operations
//! - Rate limiting may be needed for expensive operations

use serde_json::Value;
use tauri::AppHandle;

/// Invoke IPC methods.
///
/// This command accepts method invocation requests from the Wind frontend
/// and delegates them to the WindServiceHandlers for execution.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
/// * `method` - Name of the method to invoke
/// * `params` - JSON object containing method parameters
///
/// # Returns
///
/// Returns the method execution result as JSON, or an error string.
///
/// # Errors
///
/// Returns an error if:
/// - Method does not exist
/// - Method execution fails
/// - Parameters are invalid
#[tauri::command]
pub async fn MountainIPCInvoke(app_handle:AppHandle, method:String, params:Value) -> Result<Value, String> {
	// Convert params to Vec<Value> - if params is an array use it, otherwise wrap
	// in array
	let args = if params.is_array() {
		serde_json::from_value(params).map_err(|e| format!("Invalid params array: {}", e))?
	} else {
		vec![params]
	};

	crate::IPC::WindServiceHandlers::mountain_ipc_invoke(app_handle, method, args).await
}
