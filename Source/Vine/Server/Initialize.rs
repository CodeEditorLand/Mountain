//! # Initialize (Vine Server)
//!
//! Contains the logic to initialize and start the Mountain gRPC server.
//!
//! This module provides the entry point for starting Vine's gRPC servers:
//! - **MountainServiceServer**: Listens for connections from Cocoon sidecar
//! - **CocoonServiceServer**: Listens for connections from Mountain
//!   (bidirectional)
//!
//! ## Initialization Process
//!
//! 1. Validates socket addresses
//! 2. Retrieves ApplicationRunTime from Tauri state
//! 3. Creates service implementations with runtime dependencies
//! 4. Spawns server tasks as background tokio tasks
//! 5. Servers begin listening on specified ports
//!
//! ## Server Configuration
//!
//! - **Mountaln Service**: Typically on port 50051 (configurable)
//! - **Cocoon Service**: Typically on port 50052 (configurable)
//! - Both servers support compression and message size limits
//!
//! ## Error Handling
//!
//! Initialization failures are logged and returned to the caller.
//! Once started, servers run independently and log their own errors.
//!
//! ## Lifecycle
//!
//! Servers run as detached tokio tasks. They will:
//! - Start immediately when spawned
//! - Continue until process termination or tokio runtime shutdown
//! - Log errors to the logging system
//! - Not automatically restart on failure (caller should implement retry logic
//!   if needed)

use std::{net::SocketAddr, sync::Arc};

use tauri::{AppHandle, Manager};
use tonic::transport::Server;
use ::Vine::Error::VineError;

use super::MountainVinegRPCService::MountainVinegRPCService;
use crate::{
	RPC::CocoonService::CocoonServiceImpl,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

use ::Vine::Generated::{cocoon_service_server::CocoonServiceServer, mountain_service_server::MountainServiceServer};

/// Server configuration constants
mod ServerConfig {

	use std::time::Duration;

	/// Default port for MountainService server
	pub const DEFAULT_MOUNTAIN_PORT:u16 = 50051;

	/// Default port for CocoonService server
	pub const DEFAULT_COCOON_PORT:u16 = 50052;

	/// Maximum concurrent connections per server
	pub const MAX_CONNECTIONS:usize = 100;

	/// Connection timeout duration
	pub const CONNECTION_TIMEOUT:Duration = Duration::from_secs(30);

	/// Default message size limit (4MB)
	pub const MAX_MESSAGE_SIZE:usize = 4 * 1024 * 1024;
}

/// Validates a socket address string before parsing.
///
/// # Parameters
/// - `AddressString`: The address string to validate
/// - `ServerName`: Name of the server for error messages
///
/// # Returns
/// - `Ok(SocketAddr)`: Validated and parsed socket address
/// - `Err(VineError)`: Invalid address format
fn ValidateSocketAddress(AddressString:&str, ServerName:&str) -> Result<SocketAddr, VineError> {
	if AddressString.is_empty() {
		return Err(VineError::InvalidMessageFormat(format!(
			"{} address cannot be empty",
			ServerName
		)));
	}

	if AddressString.len() > 256 {
		return Err(VineError::InvalidMessageFormat(format!(
			"{} address exceeds maximum length (256 characters)",
			ServerName
		)));
	}

	match AddressString.parse::<SocketAddr>() {
		Ok(addr) => {
			// Validate port is within valid range
			if addr.port() < 1024 {
				dev_log!(
					"grpc",
					"warn: [VineServer] {} using privileged port {}, this may require elevated privileges",
					ServerName,
					addr.port()
				);
			}

			Ok(addr)
		},

		Err(e) => Err(VineError::AddressParseError(e)),
	}
}

/// Initializes and starts the gRPC servers on background tasks.
///
/// This function retrieves the core `ApplicationRunTime` from Tauri's managed
/// state, instantiates the gRPC service implementations
/// (`MountainVinegRPCService` and `CocoonServiceServer`), and uses `tonic` to
/// serve them at the specified addresses.
///
/// # Parameters
/// - `ApplicationHandle`: The Tauri application handle
/// - `MountainAddressString`: The address and port to bind the Mountain server
///   to (e.g., `"[::1]:50051"`)
/// - `CocoonAddressString`: The address and port to bind the Cocoon server to
///   (e.g., `"[::1]:50052"`)
///
/// # Returns
/// - `Ok(())`: Successfully initialized and started both servers
/// - `Err(VineError)`: Initialization failed (invalid address, missing runtime,
///   etc.)
///
/// # Errors
///
/// This function will return an error if:
/// - Either socket address string is invalid or unparseable
/// - ApplicationRunTime is not available in Tauri state
/// - Server task spawning fails (rare)
///
/// # Example
///
/// ```rust,no_run
/// # use Vine::Server::Initialize::Initialize;
/// # use tauri::AppHandle;
/// # async fn example(handle: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
/// Initialize(handle, "[::1]:50051".to_string(), "[::1]:50052".to_string())?;
/// # Ok(())
/// # }
/// ```
///
/// # Notes
///
/// - Servers run as detached tokio tasks
/// - Initialization is async-safe but function is synchronous
/// - Servers log errors independently after startup
/// - Use `Default` addresses for development (localhost with default ports)
pub fn Initialize(
	ApplicationHandle:AppHandle,

	MountainAddressString:String,

	CocoonAddressString:String,
) -> Result<(), VineError> {
	dev_log!("grpc", "[VineServer] Initializing Vine gRPC servers...");

	crate::dev_log!("grpc", "initializing Vine gRPC servers");

	// Validate and parse socket addresses
	let MountainAddress = ValidateSocketAddress(&MountainAddressString, "MountainService")?;

	let CocoonAddress = ValidateSocketAddress(&CocoonAddressString, "CocoonService")?;

	dev_log!("grpc", "[VineServer] MountainService will bind to: {}", MountainAddress);

	dev_log!(
		"grpc",
		"[VineServer] Cocoon expected on: {} (started by Cocoon process)",
		CocoonAddress
	);

	crate::dev_log!("grpc", "Mountain={} Cocoon(remote)={}", MountainAddress, CocoonAddress);

	// Retrieve ApplicationRunTime from Tauri managed state
	let RunTime = ApplicationHandle
		.try_state::<Arc<ApplicationRunTime>>()
		.ok_or_else(|| {
			let msg = "[VineServer] CRITICAL: ApplicationRunTime not found in Tauri state. Server cannot start.";

			dev_log!("grpc", "error: {}", msg);

			VineError::InternalLockError(msg.to_string())
		})?
		.inner()
		.clone();

	dev_log!("grpc", "[VineServer] ApplicationRunTime retrieved successfully");

	// Create MountainService implementation (handles calls from Cocoon to Mountain)
	let MountainService = MountainVinegRPCService::Create(ApplicationHandle.clone(), RunTime.clone());

	// Create CocoonService implementation (handles calls from Mountain to Cocoon)
	let cocoon_service_impl = CocoonServiceImpl::new(RunTime.Environment.clone());

	dev_log!("grpc", "[VineServer] Service implementations created");

	// Spawn Mountain server to run in the background
	let MountainServerName = MountainAddress.to_string();

	tokio::spawn(async move {
		dev_log!(
			"grpc",
			"[VineServer] Starting MountainService gRPC server on {}",
			MountainServerName
		);

		let ServerResult = Server::builder()
			.add_service(
				MountainServiceServer::new(MountainService)
					.max_decoding_message_size(ServerConfig::MAX_MESSAGE_SIZE)
					.max_encoding_message_size(ServerConfig::MAX_MESSAGE_SIZE),
			)
			.serve(MountainAddress)
			.await;

		match ServerResult {
			Ok(_) => {
				dev_log!("grpc", "[VineServer] MountainService server shut down gracefully");
			},
			Err(e) => {
				dev_log!("grpc", "error: [VineServer] MountainService gRPC server error: {}", e);
			},
		}
	});

	// NOTE: CocoonService gRPC server is NOT started by Mountain.
	// Port 50052 is reserved for Cocoon's own gRPC server (started by
	// Cocoon's Effect-TS bootstrap, Stage 5). Mountain connects to Cocoon
	// as a CLIENT on 50052 via Vine::Client::ConnectToSideCar.
	// Starting CocoonServiceServer here would cause EADDRINUSE when Cocoon
	// tries to bind the same port.
	let _ = cocoon_service_impl; // suppress unused variable warning

	dev_log!(
		"grpc",
		"[VineServer] MountainService gRPC server initialized on {}",
		MountainAddress
	);

	Ok(())
}
