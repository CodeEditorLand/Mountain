// File: Mountain/Source/Air/AirClient.rs
// Role: gRPC client wrapper for Air daemon service
// Responsibilities:
//   - Manage gRPC connection to Air service
//   - Implement all Air service methods
//   - Translate tonic errors to CommonError
//   - Provide connection retry capabilities (optional)

use std::{collections::HashMap, time::Duration};

use CommonLibrary::Error::CommonError::CommonError;
use log::{debug, error, info, warn};
use tonic::transport::{Channel, Endpoint};

/// Default gRPC server address for the Air daemon.
///
/// Port Allocation:
/// - 50051: Mountain Vine server
/// - 50052: Cocoon Vine server (VS Code extension hosting)
/// - 50053: Air Vine server (Air daemon services - authentication, updates, and
///   more)
pub const DEFAULT_AIR_SERVER_ADDRESS:&str = "[::1]:50053";

/// Air gRPC client wrapper that handles connection to the Air daemon service.
/// This provides a clean interface for Mountain to interact with Air's
/// capabilities including update management, authentication, file indexing,
/// and system monitoring.
#[derive(Debug, Clone)]
pub struct AirClient {
	/// The underlying tonic gRPC client (commented out until AirIntegration
	/// feature)
	inner:Option<()>,
	/// Address of the Air daemon
	address:String,
}

impl AirClient {
	/// Creates a new AirClient and connects to the Air daemon service.
	///
	/// # Arguments
	/// * `address` - The gRPC server address (e.g., "http://[::1]:50053")
	///
	/// # Returns
	/// * `Ok(Self)` - Successfully created client
	/// * `Err(CommonError)` - Connection failure with descriptive error
	///
	/// # TODO
	/// Implement actual gRPC connection when AirIntegration feature is ready
	pub async fn new(address:&str) -> Result<Self, CommonError> {
		info!("[AirClient] Creating Air client (connection deferred until AirIntegration feature)");

		Ok(Self { inner:None, address:address.to_string() })
	}

	/// Checks if the client is connected to the Air daemon.
	///
	/// # Returns
	/// * `true` - Client is connected
	/// * `false` - Client is not connected
	pub fn is_connected(&self) -> bool { self.inner.is_some() }

	/// Gets the address of the Air daemon.
	///
	/// # Returns
	/// The address string
	pub fn address(&self) -> &str { &self.address }
}
