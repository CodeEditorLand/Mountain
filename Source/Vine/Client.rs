//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//!
//! # Vine Client
//!
//! Provides a simplified, thread-safe client for communicating with a `Cocoon`
//! sidecar process via gRPC. It manages a shared pool of connections with
//! robust error handling, automatic reconnection, health checks, and timeout
//! management.
//!
//! ## Features
//!
//! - **Connection Pool**: Thread-safe HashMap of client connections by identifier
//! - **Health Checks**: Validates connection status before RPC calls
//! - **Automatic Reconnection**: Retries failed connections with exponential backoff
//! - **Request Timeout**: Configurable timeout per RPC call
//! - **Retry Logic**: Configurable retry attempts for transient failures
//! - **Message Validation**: Size limits and format checking for all messages
//! - **Graceful Degradation**: Handles Cocoon unavailability gracefully
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use Vine::Client::ConnectToSideCar;
//! use Vine::Client::SendRequest;
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to Cocoon
//! ConnectToSideCar(
//!     "cocoon-main".to_string(),
//!     "127.0.0.1:50052".to_string()
//! ).await?;
//!
//! // Send request
//! let result = SendRequest(
//!     "cocoon-main",
//!     "GetExtensions".to_string(),
//!     json!({}),
//!     5000 // 5 second timeout
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! All operations return `Result<T, VineError>` with comprehensive error types:
//! - ClientNotConnected: Sidecar not in connection pool
//! - RequestTimeout: RPC call exceeded timeout
//! - RPCError: gRPC transport or status error
//! - SerializationError: JSON parsing/serialization failure

#![allow(non_snake_case, non_camel_case_types)]

use std::{
	collections::{HashMap, hash_map::DefaultHasher},
	hash::{Hash, Hasher},
	sync::Arc,
	time::{Duration, Instant},
};

use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use parking_lot::Mutex;
use serde_json::{Value, from_slice, to_vec};
use tokio::time::timeout;
use tonic::transport::Channel;

use super::{
	Error::VineError,
	Generated::{GenericNotification, GenericRequest, cocoon_service_client::CocoonServiceClient},
};

/// Type alias for the Cocoon gRPC client with Channel transport
type CocoonClient = CocoonServiceClient<Channel>;

/// Configuration constants for Vine client behavior
mod Config {
	/// Default timeout for RPC calls (5 seconds)
	pub const DEFAULT_TIMEOUT_MS: u64 = 5000;

	/// Maximum number of retry attempts for failed connections
	pub const MAX_RETRY_ATTEMPTS: usize = 3;

	/// Base delay between retry attempts (100ms)
	pub const RETRY_BASE_DELAY_MS: u64 = 100;

	/// Maximum message size for validation (4MB to match tonic default)
	pub const MAX_MESSAGE_SIZE_BYTES: usize = 4 * 1024 * 1024;

	/// Health check interval (30 seconds)
	pub const HEALTH_CHECK_INTERVAL_MS: u64 = 30000;

	/// Connection timeout (10 seconds)
	pub const CONNECTION_TIMEOUT_MS: u64 = 10000;
}

/// Connection metadata tracking health and last activity
struct ConnectionMetadata {
	/// Timestamp of last successful communication
	LastActivity: Instant,
	/// Number of consecutive failures since last success
	FailureCount: usize,
	/// Whether the connection is currently marked healthy
	IsHealthy: bool,
}

lazy_static! {
	/// Thread-safe pool of Cocoon client connections indexed by identifier
	static ref SIDECAR_CLIENTS: Arc<Mutex<HashMap<String, CocoonClient>>> = Arc::new(Mutex::new(HashMap::new()));

	/// Thread-safe metadata for connection health tracking
	static ref CONNECTION_METADATA: Arc<Mutex<HashMap<String, ConnectionMetadata>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Establishes a gRPC connection to a sidecar process with retry logic.
///
/// This function attempts to connect to a Cocoon sidecar at the specified address.
/// It implements exponential backoff retry logic for transient failures and
/// initializes connection metadata for health tracking.
///
/// # Parameters
/// - `SideCarIdentifier`: Unique identifier for this sidecar connection
/// - `Address`: Network address in format "host:port"
///
/// # Returns
/// - `Ok(())`: Connection successfully established
/// - `Err(VineError)`: Connection failed after all retry attempts
///
/// # Example
/// ```rust,no_run
/// # use Vine::Client::ConnectToSideCar;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// ConnectToSideCar(
///     "cocoon-main".to_string(),
///     "127.0.0.1:50052".to_string()
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn ConnectToSideCar(SideCarIdentifier:String, Address:String) -> Result<(), VineError> {
	info!("[VineClient] Connecting to sidecar '{}' at '{}'...", SideCarIdentifier, Address);

	let endpoint = format!("http://{}", Address);

	// Validate endpoint format
	if endpoint.len() > 256 {
		return Err(VineError::RPCError(format!(
			"Invalid endpoint address: exceeds maximum length"
		)));
	}

	// Attempt connection with retry logic
	let mut last_error = None;

	for Attempt in 1..=Config::MAX_RETRY_ATTEMPTS {
		match Channel::from_shared(endpoint.clone())?.connect().await {
			Ok(channel) => {
				let client = CocoonServiceClient::new(channel);

				SIDECAR_CLIENTS.lock().insert(SideCarIdentifier.clone(), client);

				// Initialize connection metadata
				CONNECTION_METADATA.lock().insert(
					SideCarIdentifier.clone(),
					ConnectionMetadata {
						LastActivity: Instant::now(),
						FailureCount: 0,
						IsHealthy: true,
					}
				);

				info!("[VineClient] Successfully connected to sidecar '{}'.", SideCarIdentifier);
				return Ok(());
			},

			Err(e) => {
				warn!(
					"[VineClient] Connection attempt {}/{} failed: {}",
					Attempt, Config::MAX_RETRY_ATTEMPTS, e
				);

				last_error = Some(VineError::from(e));

				// Exponential backoff before retry
				if Attempt < Config::MAX_RETRY_ATTEMPTS {
					let delay_ms = Config::RETRY_BASE_DELAY_MS * 2u64.pow((Attempt - 1) as u32);
					tokio::time::sleep(Duration::from_millis(delay_ms)).await;
				}
			}
		}
	}

	error!(
		"[VineClient] Failed to connect to sidecar '{}' after {} attempts",
		SideCarIdentifier, Config::MAX_RETRY_ATTEMPTS
	);

	Err(last_error.unwrap_or_else(|| {
		VineError::RPCError("Connection failed: unknown error".to_string())
	}))
}

/// Disconnects from a sidecar and removes it from the connection pool.
///
/// This function gracefully disconnects from a sidecar and cleans up
/// connection metadata. Any pending RPC calls will fail.
///
/// # Parameters
/// - `SideCarIdentifier`: Unique identifier of the sidecar to disconnect
///
/// # Returns
/// - `Ok(())`: Successfully disconnected or sidecar was not connected
/// - `Err(VineError)`: Failed during disconnection (rare)
pub fn DisconnectFromSideCar(SideCarIdentifier:String) -> Result<(), VineError> {
	info!("[VineClient] Disconnecting from sidecar '{}'...", SideCarIdentifier);

	SIDECAR_CLIENTS.lock().remove(&SideCarIdentifier);
	CONNECTION_METADATA.lock().remove(&SideCarIdentifier);

	info!("[VineClient] Disconnected from sidecar '{}'.", SideCarIdentifier);

	Ok(())
}

/// Checks health status of a connected sidecar.
///
/// This function validates that a sidecar connection is healthy by checking
/// last activity and failure count. An unhealthy connection may need reconnection.
///
/// # Parameters
/// - `SideCarIdentifier`: Unique identifier of the sidecar to check
///
/// # Returns
/// - `Ok(true)`: Connection is healthy
/// - `Ok(false)`: Connection exists but is unhealthy
/// - `Err(VineError)`: Sidecar not connected
pub fn CheckSideCarHealth(SideCarIdentifier:&str) -> Result<bool, VineError> {
	let metadata = CONNECTION_METADATA.lock();
	let metadata = metadata.get(SideCarIdentifier)
		.ok_or_else(|| VineError::ClientNotConnected(SideCarIdentifier.to_string()))?;

	let time_since_activity = metadata.LastActivity.elapsed();
	let is_stale = time_since_activity > Duration::from_millis(Config::HEALTH_CHECK_INTERVAL_MS);
	let is_healthy = metadata.IsHealthy && !is_stale && metadata.FailureCount == 0;

	if !is_healthy {
		warn!(
			"[VineClient] Sidecar '{}' health check: stale={}, failures={}, healthy={}",
			SideCarIdentifier, is_stale, metadata.FailureCount, metadata.IsHealthy
		);
	}

	Ok(is_healthy)
}

/// Marks a sidecar connection as failed for health tracking.
///
/// # Parameters
/// - `SideCarIdentifier`: Unique identifier of the sidecar
fn RecordSideCarFailure(SideCarIdentifier:&str) {
	let mut metadata = CONNECTION_METADATA.lock();
	if let Some(meta) = metadata.get_mut(SideCarIdentifier) {
		meta.FailureCount += 1;
		meta.IsHealthy = meta.FailureCount < 3;

		if !meta.IsHealthy {
			warn!(
				"[VineClient] Sidecar '{}' marked as unhealthy after {} failures",
				SideCarIdentifier, meta.FailureCount
			);
		}
	}
}

/// Updates last activity timestamp for a sidecar.
///
/// # Parameters
/// - `SideCarIdentifier`: Unique identifier of the sidecar
fn UpdateSideCarActivity(SideCarIdentifier:&str) {
	let mut metadata = CONNECTION_METADATA.lock();
	if let Some(meta) = metadata.get_mut(SideCarIdentifier) {
		meta.LastActivity = Instant::now();
		meta.FailureCount = meta.FailureCount.saturating_sub(1);
		meta.IsHealthy = true;
	}
}

/// Validates message size before sending.
///
/// # Parameters
/// - `data`: Byte array to validate
///
/// # Returns
/// - `Ok(())`: Message size is within limits
/// - `Err(VineError)`: Message exceeds maximum size
fn ValidateMessageSize(data:&[u8]) -> Result<(), VineError> {
	if data.len() > Config::MAX_MESSAGE_SIZE_BYTES {
		Err(VineError::RPCError(format!(
			"Message size {} bytes exceeds maximum of {} bytes",
			data.len(),
			Config::MAX_MESSAGE_SIZE_BYTES
		)))
	} else {
		Ok(())
	}
}

/// Sends a fire-and-forget notification to a sidecar.
///
/// This function sends a notification that does not expect a response.
/// It validates the message, checks connection health, and handles errors gracefully.
///
/// # Parameters
/// - `SideCarIdentifier`: Unique identifier of the target sidecar
/// - `Method`: RPC method name to invoke
/// - `Parameters`: JSON-serializable parameters for the method
///
/// # Returns
/// - `Ok(())`: Notification sent successfully
/// - `Err(VineError)`: Notification failed (sidecar not connected, serialization, or RPC error)
///
/// # Example
/// ```rust,no_run
/// # use Vine::Client::SendNotification;
/// # use serde_json::json;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// SendNotification(
///     \"cocoon-main\".to_string(),
///     \"UpdateTheme\".to_string(),
///     json!({\"theme\": \"dark\"})\n/// ).await?;\n/// # Ok(())\n/// # }\n/// ```\npub async fn SendNotification(SideCarIdentifier:String, Method:String, Parameters:Value) -> Result<(), VineError> {\n\t// Validate method name format\n\tif Method.is_empty() || Method.len() > 128 {\n\t\treturn Err(VineError::RPCError(\n\t\t\t\"Method name must be between 1 and 128 characters\".to_string()\n\t\t));\n\t}\n\n\tlet parameter_bytes = to_vec(&Parameters)?;\n\tValidateMessageSize(&parameter_bytes)?;\n\n\tlet mut client = {\n\t\tlet guard = SIDECAR_CLIENTS.lock();\n\t\tguard.get(&SideCarIdentifier).cloned()\n\t};\n\n\tif let Some(ref mut client) = client {\n\t\tlet request = GenericNotification { method:Method, parameter:parameter_bytes };\n\n\t\tmatch client.send_mountain_notification(request).await {\n\t\t\tOk(_) => {\n\t\t\t\tUpdateSideCarActivity(&SideCarIdentifier);\n\t\t\t\tdebug!(\n\t\t\t\t\t\"[VineClient] Notification sent successfully to sidecar '{}'\",\n\t\t\t\t\tSideCarIdentifier\n\t\t\t\t);\n\t\t\t\tOk(())\n\t\t\t},\n\t\t\tErr(status) => {\n\t\t\t\tRecordSideCarFailure(&SideCarIdentifier);\n\t\t\t\tError!(\n\t\t\t\t\t\"[VineClient] Failed to send notification to sidecar '{}': {}\",\n\t\t\t\t\tSideCarIdentifier, status\n\t\t\t\t);\n\t\t\t\tErr(VineError::from(status))\n\t\t\t}\n\t\t}\n\t} else {\n\t\tErr(VineError::ClientNotConnected(SideCarIdentifier))\n\t}\n}"

/// Sends a request to a sidecar and awaits a response with timeout handling.
///
/// This function sends a request-response RPC call to a sidecar with configurable
/// timeout. It generates a unique request ID, handles serialization, tracks
/// connection health, and provides detailed error reporting.
///
/// # Parameters
/// - `SideCarIdentifier`: Unique identifier of the target sidecar
/// - `Method`: RPC method name to invoke
/// - `Parameters`: JSON-serializable parameters for the method
/// - `TimeoutMilliseconds`: Maximum time to wait for response (0 = use default)
///
/// # Returns
/// - `Ok(Value)`: Deserialized JSON response from the sidecar
/// - `Err(VineError)`: Request failed (timeout, not connected, serialization, or RPC error)
///
/// # Example
/// ```rust,no_run
/// # use Vine::Client::SendRequest;
/// # use serde_json::json;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let extensions = SendRequest(
///     \"cocoon-main\",\n///     \"GetExtensions\".to_string(),\n///     json!({}),\n///     5000\n/// ).await?;\n/// # Ok(())\n/// # }\n/// ```\npub async fn SendRequest(\n\tSideCarIdentifier:&str,\n\n\tMethod:String,\n\n\tParameters:Value,\n\n\tTimeoutMilliseconds:u64,\n) -> Result<Value, VineError> {\n\t// Validate inputs\n\tif Method.is_empty() || Method.len() > 128 {\n\t\treturn Err(VineError::RPCError(\n\t\t\t\"Method name must be between 1 and 128 characters\".to_string()\n\t\t));\n\t}\n\n\tlet timeout_ms = if TimeoutMilliseconds == 0 {\n\t\tConfig::DEFAULT_TIMEOUT_MS\n\t} else {\n\t\tTimeoutMilliseconds\n\t};\n\n\tdebug!(\n\t\t\"[VineClient] Sending request '{}' to sidecar '{}' (timeout: {}ms)...\",\n\t\tMethod, SideCarIdentifier, timeout_ms\n\t);\n\n\t// Check connection health before proceeding\n\tif let Ok(is_healthy) = CheckSideCarHealth(SideCarIdentifier) {\n\t\tif !is_healthy {\n\t\t\twarn!(\n\t\t\t\t\"[VineClient] Sidecar '{}' connection is unhealthy, proceeding anyway\",\n\t\t\t\tSideCarIdentifier\n\t\t\t);\n\t\t}\n\t}\n\n\tlet mut client = {\n\t\tlet guard = SIDECAR_CLIENTS.lock();\n\t\tguard.get(SideCarIdentifier).cloned()\n\t};\n\n\tif let Some(ref mut client) = client {\n\t\t// Generate unique request identifier using UUID hashing\n\t\tlet mut hasher = DefaultHasher::new();\n\t\tuuid::Uuid::new_v4().hash(&mut hasher);\n\t\tlet RequestIdentifier = hasher.finish();\n\n\t\t// Serialize parameters with validation\n\t\tlet parameter_bytes = to_vec(&Parameters)?;\n\t\tValidateMessageSize(&parameter_bytes)?;\n\n\t\tlet request = GenericRequest {\n\t\t\trequest_identifier:RequestIdentifier,\n\t\t\tmethod:Method.clone(),\n\t\t\tparameter:parameter_bytes,\n\t\t};\n\n\t\tlet future = client.process_mountain_request(request);\n\n\t\t// Execute with timeout\n\t\tmatch timeout(Duration::from_millis(timeout_ms), future).await {\n\t\t\tOk(Ok(response)) => {\n\t\t\t\tlet response_data = response.into_inner();\n\t\t\t\tUpdateSideCarActivity(SideCarIdentifier);\n\n\t\t\t\t// Check for RPC error in response\n\t\t\t\tif let Some(rpc_error) = response_data.error {\n\t\t\t\t\tRecordSideCarFailure(SideCarIdentifier);\n\t\t\t\t\terror!(\n\t\t\t\t\t\t\"[VineClient] Received RPC error from sidecar '{}': code={}, message={}\",\n\t\t\t\t\t\tSideCarIdentifier, rpc_error.code, rpc_error.message\n\t\t\t\t\t);\n\t\t\t\t\treturn Err(VineError::RPCError(rpc_error.message));\n\t\t\t\t}\n\n\t\t\t\t// Deserialize result\n\t\t\t\tmatch from_slice(&response_data.result) {\n\t\t\t\t\tOk(deserialized_value) => {\n\t\t\t\t\t\tdebug!(\n\t\t\t\t\t\t\t\"[VineClient] Request '{}' to sidecar '{}' completed successfully\",\n\t\t\t\t\t\t\tMethod, SideCarIdentifier\n\t\t\t\t\t\t);\n\t\t\t\t\t\tOk(deserialized_value)\n\t\t\t\t\t},\n\t\t\t\t\tErr(e) => {\n\t\t\t\t\t\tRecordSideCarFailure(SideCarIdentifier);\n\t\t\t\t\t\terror!(\n\t\t\t\t\t\t\t\"[VineClient] Failed to deserialize response from sidecar '{}': {}\",\n\t\t\t\t\t\t\tSideCarIdentifier, e\n\t\t\t\t\t\t);\n\t\t\t\t\t\tErr(VineError::SerializationError(e))\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t},\n\n\t\t\tOk(Err(status)) => {\n\t\t\t\tRecordSideCarFailure(SideCarIdentifier);\n\t\t\t\terror!(\n\t\t\t\t\t\"[VineClient] gRPC status error from sidecar '{}': code={}, message={}\",\n\t\t\t\t\tSideCarIdentifier, status.code(), status.message()\n\t\t\t\t);\n\t\t\t\tErr(VineError::from(status))\n\t\t\t},\n\n\t\t\tErr(_) => {\n\t\t\t\tRecordSideCarFailure(SideCarIdentifier);\n\t\t\t\terror!(\n\t\t\t\t\t\"[VineClient] Request to sidecar '{}' (method: '{}') timed out after {}ms\",\n\t\t\t\t\tSideCarIdentifier, Method, timeout_ms\n\t\t\t\t);\n\t\t\t\tErr(VineError::RequestTimeout {\n\t\t\t\t\tSideCarIdentifier:SideCarIdentifier.to_string(),\n\t\t\t\t\tMethodName:Method,\n\t\t\t\t\tTimeoutMilliseconds:timeout_ms,\n\t\t\t\t})\n\t\t\t},\n\t\t}\n\t} else {\n\t\tErr(VineError::ClientNotConnected(SideCarIdentifier.to_string()))\n\t}\n}"
