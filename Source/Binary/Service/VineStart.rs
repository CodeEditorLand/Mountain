//! # Vine Start Module
//!
//! Initializes and starts the Vine gRPC server.

/// Starts the Vine gRPC server at the specified addresses.
///
/// # Arguments
///
/// * `ApplicationHandle` - The Tauri application handle
/// * `PrimaryAddress` - The primary server address (e.g., "\\[::1\\]:50051")
/// * `SecondaryAddress` - The secondary server address (e.g.,
///   "\\[::1\\]:50052")
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # Vine Server Functionality
///
/// The Vine gRPC server provides:
/// - Inter-service communication infrastructure
/// - gRPC method handling for various services
/// - Multi-port support for different service types
///
/// # Addresses
///
/// - Primary: `[::1]:50051` - Main service communication
/// - Secondary: `[::1]:50052` - Auxiliary service communication
///
/// # Errors
///
/// Returns an error if Vine server initialization fails.
pub async fn VineStart(
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
