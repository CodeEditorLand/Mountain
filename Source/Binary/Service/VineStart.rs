//! # Vine Start Module
//!
//! Initializes and starts the Vine gRPC server.

/// Starts the Vine gRPC server at the specified primary and secondary
/// addresses.
///
/// # Parameters
///
/// - `ApplicationHandle` — Tauri application handle used for service
///   registration.
/// - `PrimaryAddress` — Primary server address, e.g. `[::1]:50051`.
/// - `SecondaryAddress` — Secondary server address, e.g. `[::1]:50052`.
///
/// # Returns
///
/// `Ok(())` on success, or `Err(String)` if server initialization fails.
///
/// # Errors
///
/// Returns an error if the Vine gRPC server fails to bind or initialize on
/// either address.
pub async fn Fn(
	ApplicationHandle:tauri::AppHandle,

	PrimaryAddress:String,

	SecondaryAddress:String,
) -> Result<(), String> {
	match crate::Vine::Server::Initialize::Initialize(ApplicationHandle, PrimaryAddress, SecondaryAddress) {
		Ok(()) => {
			dev_log!("grpc", "[Vine] [Start] Vine gRPC server started successfully.");

			Ok(())
		},

		Err(e) => {
			dev_log!("grpc", "error: [Vine] [Start] Failed to start: {}", e);

			Err(format!("Failed to start Vine gRPC server: {}", e))
		},
	}
}

use crate::dev_log;
