//! # Vine Client
//!
//! Provides a simplified, thread-safe client for communicating with a `Cocoon`
//! sidecar process via gRPC. It manages a shared pool of connections with
//! robust error handling, automatic reconnection, health checks, and timeout
//! management.
//!
//! ## Features
//!
//! - **Connection Pool**: Thread-safe HashMap of client connections by
//!   identifier
//! - **Health Checks**: Validates connection status before RPC calls
//! - **Automatic Reconnection**: Retries failed connections with exponential
//!   backoff
//! - **Request Timeout**: Configurable timeout per RPC call
//! - **Retry Logic**: Configurable retry attempts for transient failures
//! - **Message Validation**: Size limits and format checking for all messages
//! - **Graceful Degradation**: Handles Cocoon unavailability gracefully
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use Vine::Client::{ConnectToSideCar, SendRequest};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to Cocoon
//! ConnectToSideCar("cocoon-main".to_string(), "127.0.0.1:50052".to_string()).await?;
//!
//! // Send request
//! let result = SendRequest(
//! 	"cocoon-main",
//! 	"GetExtensions".to_string(),
//! 	json!({}),
//! 	5000, // 5 second timeout
//! )
//! .await?;
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

use std::{
	collections::HashMap,
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
	time::{Duration, Instant},
};

use lazy_static::lazy_static;
use parking_lot::Mutex;
use serde_json::{Value, from_slice, to_vec};
use tokio::time::timeout;

use super::{
	Error::VineError,
	Generated::{GenericNotification, GenericRequest, cocoon_service_client::CocoonServiceClient},
};
use crate::dev_log;

/// Type alias for the Cocoon gRPC client with Channel transport
type CocoonClient = CocoonServiceClient<tonic::transport::Channel>;

/// Configuration constants for Vine client behavior
mod Config {
	/// Default timeout for RPC calls (5 seconds)
	pub const DEFAULT_TIMEOUT_MS:u64 = 5000;

	/// Maximum number of retry attempts for failed connections
	pub const MAX_RETRY_ATTEMPTS:usize = 3;

	/// Base delay between retry attempts (100ms)
	pub const RETRY_BASE_DELAY_MS:u64 = 100;

	/// Maximum message size for validation (4MB to match tonic default)
	pub const MAX_MESSAGE_SIZE_BYTES:usize = 4 * 1024 * 1024;

	/// Health check interval (30 seconds)
	pub const HEALTH_CHECK_INTERVAL_MS:u64 = 30000;

	/// Connection timeout (10 seconds)
	#[allow(dead_code)]
	pub const CONNECTION_TIMEOUT_MS:u64 = 10000;
}

/// Connection metadata tracking health and last activity
struct ConnectionMetadata {
	/// Timestamp of last successful communication
	LastActivity:Instant,
	/// Number of consecutive failures since last success
	FailureCount:usize,
	/// Whether the connection is currently marked healthy
	IsHealthy:bool,
}

lazy_static! {
	/// Thread-safe pool of Cocoon client connections indexed by identifier
	static ref SIDECAR_CLIENTS: Arc<Mutex<HashMap<String, CocoonClient>>> = Arc::new(Mutex::new(HashMap::new()));

	/// Thread-safe metadata for connection health tracking
	static ref CONNECTION_METADATA: Arc<Mutex<HashMap<String, ConnectionMetadata>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Process-wide shutdown flag. Set to `true` once Mountain has issued
/// `$shutdown` (or SIGKILL'd) the Cocoon sidecar. After this point any
/// further `SendNotification` / `SendRequest` calls short-circuit
/// instead of attempting a TCP connect to a dead socket and logging a
/// false-positive error. Background tasks (PTY reader, file watcher,
/// diagnostics emitter) drain naturally when the gRPC layer becomes a
/// no-op.
static SHUTDOWN_FLAG:AtomicBool = AtomicBool::new(false);

/// Mark the gRPC client as shutting down. Called from
/// `RunTime::Shutdown::ShutdownCocoonWithRetry` immediately before
/// `HardKillCocoon` so any inflight notification attempted after the
/// SIGKILL window returns silently with `Ok(())` instead of logging
/// a `Connection refused` error.
pub fn MarkShutdown() { SHUTDOWN_FLAG.store(true, Ordering::Relaxed); }

/// Whether the gRPC client has been marked shutting down.
pub fn IsShuttingDown() -> bool { SHUTDOWN_FLAG.load(Ordering::Relaxed) }

// ============================================================================
// LAND-PATCH B7-S6: notification fan-out broadcast.
//
// Today every `SendNotification` is fire-and-forget; the caller
// gets `Result<(), VineError>` and that's it. There is no way for
// other parts of Mountain (Effect-TS supervisors, dev log, OTel
// span emitter, future Mist WebSocket bridge) to *also* observe
// the notification flow without each one wiring a parallel
// callback at the call site.
//
// `tokio::sync::broadcast` solves this: every `SendNotification`
// also publishes a `NotificationFrame` on a single shared channel.
// Subscribers get a `Receiver<NotificationFrame>` and consume at
// their own pace; lagging subscribers see `RecvError::Lagged(n)`
// with `n` = dropped frames (drop-oldest semantics, capacity 4096).
//
// This is the structural foundation for the streaming-gRPC
// migration (Patch 14): once the bidirectional `OpenChannel*`
// streams land, the multiplexer's notification-frame reader feeds
// THIS broadcast channel directly. Subscribers don't move.
//
// ============================================================================

/// One observed notification frame fan-out from `SendNotification`
/// or (future) the streaming-channel multiplexer.
#[derive(Debug, Clone)]
pub struct NotificationFrame {
	pub SideCarIdentifier:String,
	pub Method:String,
	pub Parameters:Value,
	/// Monotonic process-relative nanosecond timestamp at fan-out
	/// time. Useful for OTel span correlation without burning a
	/// `SystemTime::now()` per frame.
	pub TimestampNanos:u64,
}

/// Broadcast capacity. Drop-oldest when full. 4096 covers the
/// observed worst-case storms (sky://diagnostics/changed at
/// 50-200/s during rust-analyzer cargo-check) with margin.
const NOTIFICATION_BROADCAST_CAPACITY:usize = 4096;

lazy_static! {
	static ref NOTIFICATION_BROADCAST: tokio::sync::broadcast::Sender<NotificationFrame> = {
		let (Sender, _) = tokio::sync::broadcast::channel(NOTIFICATION_BROADCAST_CAPACITY);
		Sender
	};
}

/// Subscribe to the global notification fan-out. Each call returns
/// a fresh receiver that observes every notification fanned out
/// AFTER subscribe time (broadcast semantics; no historical replay).
///
/// Use `tokio::sync::broadcast::Receiver::recv().await` to consume,
/// or `BroadcastStream::new(rx)` to adapt to a `Stream`.
///
/// Drop the receiver to unsubscribe.
pub fn SubscribeNotifications() -> tokio::sync::broadcast::Receiver<NotificationFrame> {
	NOTIFICATION_BROADCAST.subscribe()
}

/// Number of currently-active subscribers. Diagnostic; useful for
/// validating that subscribers haven't leaked.
pub fn SubscriberCount() -> usize { NOTIFICATION_BROADCAST.receiver_count() }

/// Public-crate alias for `PublishNotification` so `Vine::Multiplexer`
/// can fan out notifications received over the streaming channel
/// through the same broadcast subscribers consume from.
pub(crate) fn PublishNotificationFromMux(SideCarIdentifier:&str, Method:&str, Parameters:&Value) {
	PublishNotification(SideCarIdentifier, Method, Parameters)
}

/// Internal: publish a notification to the broadcast. Called from
/// `SendNotification` after the wire send succeeds, and from the
/// streaming multiplexer when it lands. `try_send` semantics -
/// no awaiting, no failure surfaced (a slow subscriber must not
/// stall the producer).
fn PublishNotification(SideCarIdentifier:&str, Method:&str, Parameters:&Value) {
	let Frame = NotificationFrame {
		SideCarIdentifier:SideCarIdentifier.to_string(),
		Method:Method.to_string(),
		Parameters:Parameters.clone(),
		TimestampNanos:crate::IPC::DevLog::NowNano(),
	};
	// `send` returns Err only when there are zero receivers;
	// we don't care.
	let _ = NOTIFICATION_BROADCAST.send(Frame);
}

/// Establishes a gRPC connection to a sidecar process with retry logic.
///
/// This function attempts to connect to a Cocoon sidecar at the specified
/// address. It implements exponential backoff retry logic for transient
/// failures and initializes connection metadata for health tracking.
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
/// ConnectToSideCar("cocoon-main".to_string(), "127.0.0.1:50052".to_string()).await?;
/// # Ok(())
/// # }
/// ```
pub async fn ConnectToSideCar(SideCarIdentifier:String, Address:String) -> Result<(), VineError> {
	dev_log!(
		"grpc",
		"[VineClient] Connecting to sidecar '{}' at '{}'...",
		SideCarIdentifier,
		Address
	);

	let endpoint = format!("http://{}", Address);

	// Validate endpoint format
	if endpoint.len() > 256 {
		return Err(VineError::RPCError(format!("Invalid endpoint address: exceeds maximum length")));
	}

	// Attempt connection with retry logic
	let mut last_error = None;

	for attempt in 1..=Config::MAX_RETRY_ATTEMPTS {
		let result = try_connect_single(&SideCarIdentifier, &endpoint).await;

		if result.is_ok() {
			// Initialize connection metadata
			CONNECTION_METADATA.lock().insert(
				SideCarIdentifier.clone(),
				ConnectionMetadata { LastActivity:Instant::now(), FailureCount:0, IsHealthy:true },
			);

			dev_log!("grpc", "[VineClient] Successfully connected to sidecar '{}'", SideCarIdentifier);

			return Ok(result?);
		}

		// Capture last error
		last_error = Some(result.unwrap_err());

		// Wait before retry (exponential backoff)
		if attempt < Config::MAX_RETRY_ATTEMPTS {
			let delay_ms = Config::RETRY_BASE_DELAY_MS * 2_u64.pow(attempt as u32);
			tokio::time::sleep(Duration::from_millis(delay_ms)).await;
		}
	}

	Err(last_error.unwrap_or_else(|| VineError::RPCError("Connection failed".to_string())))
}

/// Single connection attempt without retry logic
async fn try_connect_single(_SideCarIdentifier:&str, endpoint:&str) -> Result<(), VineError> {
	let endpoint_url = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
		endpoint.to_string()
	} else {
		format!("http://{}", endpoint)
	};

	// LAND-PATCH B3.P-tonic: tuned h2 transport for loopback gRPC.
	//
	// Stock tonic defaults are tuned for cross-machine traffic
	// (64 KB stream window, no h2 keepalive). On loopback to Cocoon
	// at 127.0.0.1:50052 the small windows force `WINDOW_UPDATE`
	// frames on every diagnostic batch >64 KB; rust-analyzer's full
	// diagnostic emit is regularly 200-500 KB. Bump windows to 4 MB
	// stream / 16 MB connection so a single diagnostic storm fits
	// without h2 ping-pong. Keepalive at 10s detects dead Cocoon
	// faster than the 30s default.
	let UseTuned = std::env::var("LAND_TONIC_TUNED").as_deref() != Ok("0");
	let mut Endpoint = tonic::transport::Channel::from_shared(endpoint_url)
		.map_err(|e| VineError::RPCError(format!("Failed to create channel: {}", e)))?;
	if UseTuned {
		Endpoint = Endpoint
			.tcp_nodelay(true)
			.http2_keep_alive_interval(std::time::Duration::from_secs(10))
			.keep_alive_timeout(std::time::Duration::from_secs(20))
			.http2_adaptive_window(true)
			.initial_stream_window_size(4 * 1024 * 1024)
			.initial_connection_window_size(16 * 1024 * 1024)
			.concurrency_limit(1024)
			.buffer_size(256 * 1024)
			.timeout(std::time::Duration::from_secs(30))
			.connect_timeout(std::time::Duration::from_secs(5));
	}
	let channel = Endpoint
		.connect()
		.await
		.map_err(|e| VineError::RPCError(format!("Failed to connect: {}", e)))?;

	let client = CocoonClient::new(channel);

	{
		let mut clients = SIDECAR_CLIENTS.lock();
		clients.insert(_SideCarIdentifier.to_string(), client.clone());
	}

	// LAND-PATCH B7-S6 P14.1: open the bidirectional streaming
	// multiplexer alongside the unary client. Best-effort: if the
	// streaming endpoint is unimplemented (Cocoon hasn't shipped its
	// streaming handler tree yet) we log and continue. The unary
	// path stays authoritative until `LAND_VINE_STREAMING=1` flips
	// callers to the multiplexer in P14.2/P14.3.
	let StreamingEnabled = std::env::var("LAND_VINE_STREAMING").as_deref() == Ok("1");
	if StreamingEnabled {
		let SideCarIdentifierForMux = _SideCarIdentifier.to_string();
		match super::Multiplexer::Multiplexer::Open(SideCarIdentifierForMux.clone(), client).await {
			Ok(_) => {
				dev_log!("grpc", "[VineClient] streaming multiplexer opened for sidecar '{}'", _SideCarIdentifier);
			},
			Err(Error) => {
				dev_log!(
					"grpc",
					"warn: [VineClient] streaming multiplexer open failed for '{}' ({}); falling back to unary",
					_SideCarIdentifier,
					Error
				);
			},
		}
	}

	Ok(())
}

/// Disconnects from a sidecar process and removes it from the connection pool.
///
/// This function removes the sidecar from both the connection pool and
/// connection metadata tracking.
///
/// # Parameters
/// - `SideCarIdentifier`: Unique identifier of the sidecar to disconnect
///
/// # Returns
/// - `Ok(())`: Disconnection successful
/// - `Err(VineError)`: Sidecar was not connected
///
/// # Example
/// ```rust,no_run
/// # use Vine::Client::DisconnectFromSideCar;
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// DisconnectFromSideCar("cocoon-main".to_string())?;
/// # Ok(())
/// # }
/// ```
pub fn DisconnectFromSideCar(SideCarIdentifier:String) -> Result<(), VineError> {
	let mut clients = SIDECAR_CLIENTS.lock();

	if clients.remove(&SideCarIdentifier).is_some() {
		CONNECTION_METADATA.lock().remove(&SideCarIdentifier);

		dev_log!("grpc", "[VineClient] Disconnected from sidecar '{}'", SideCarIdentifier);

		Ok(())
	} else {
		Err(VineError::ClientNotConnected(SideCarIdentifier))
	}
}

/// Checks the health status of a connected sidecar.
///
/// Health is determined by:
/// - Connection exists in the pool
/// - Last activity within health check interval
/// - Failure count below threshold
///
/// # Parameters
/// - `SideCarIdentifier`: Unique identifier of the sidecar to check
///
/// # Returns
/// - `Ok(true)`: Sidecar is healthy and responsive
/// - `Ok(false)`: Sidecar exists but may have issues
/// - `Err(VineError)`: Sidecar not connected
///
/// # Example
/// ```rust,no_run
/// # use Vine::Client::CheckSideCarHealth;
/// # fn example() -> Result<bool, Box<dyn std::error::Error>> {
/// let healthy = CheckSideCarHealth("cocoon-main")?;
/// # Ok(healthy)
/// # }
/// ```
pub fn CheckSideCarHealth(SideCarIdentifier:&str) -> Result<bool, VineError> {
	let metadata = CONNECTION_METADATA.lock();

	if let Some(conn) = metadata.get(SideCarIdentifier) {
		let is_stale = conn.LastActivity.elapsed() > Duration::from_millis(Config::HEALTH_CHECK_INTERVAL_MS);
		let has_many_failures = conn.FailureCount > Config::MAX_RETRY_ATTEMPTS;

		Ok(conn.IsHealthy && !is_stale && !has_many_failures)
	} else {
		Err(VineError::ClientNotConnected(SideCarIdentifier.to_string()))
	}
}

/// Records a failure for a sidecar connection.
///
/// Increments the failure count and marks the connection as unhealthy.
///
/// # Parameters
/// - `SideCarIdentifier`: Unique identifier of the sidecar that failed
fn RecordSideCarFailure(SideCarIdentifier:&str) {
	let mut metadata = CONNECTION_METADATA.lock();

	if let Some(conn) = metadata.get_mut(SideCarIdentifier) {
		conn.FailureCount += 1;
		conn.IsHealthy = false;
	}
}

/// Updates the last activity timestamp for a sidecar.
///
/// Called after successful operations to track liveness.
///
/// # Parameters
/// - `SideCarIdentifier`: Unique identifier of the sidecar
fn UpdateSideCarActivity(SideCarIdentifier:&str) {
	let mut metadata = CONNECTION_METADATA.lock();

	if let Some(conn) = metadata.get_mut(SideCarIdentifier) {
		conn.LastActivity = Instant::now();
		conn.FailureCount = 0;
		conn.IsHealthy = true;
	}
}

/// Validates message size against maximum allowed.
///
/// Helps prevent denial-of-service attacks via overly large messages.
///
/// # Parameters
/// - `data`: Raw byte slice to validate
///
/// # Returns
/// - `Ok(())`: Message size is within limits
/// - `Err(VineError::SerializationError)`: Message exceeds maximum size
fn ValidateMessageSize(data:&[u8]) -> Result<(), VineError> {
	if data.len() > Config::MAX_MESSAGE_SIZE_BYTES {
		Err(VineError::MessageTooLarge { ActualSize:data.len(), MaxSize:Config::MAX_MESSAGE_SIZE_BYTES })
	} else {
		Ok(())
	}
}

/// Sends a request to a sidecar and waits for a response.
///
/// This is the primary method for request-response communication with sidecars.
/// It implements timeout handling and automatic connection validation.
///
/// # Parameters
/// - `SideCarIdentifier`: Unique identifier of the target sidecar
/// - `Method`: RPC method name to call
/// - `Parameters`: JSON parameters for the RPC call
/// - `TimeoutMilliseconds`: Maximum time to wait for response (default: 5000ms)
///
/// # Returns
/// - `Ok(Value)`: JSON response from the sidecar
/// - `Err(VineError)`: Request failed or timed out
///
/// # Example
/// ```rust,no_run
/// # use Vine::Client::SendRequest;
/// use serde_json::json;
/// # async fn example() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
/// let result =
/// 	SendRequest("cocoon-main".to_string(), "GetExtensions".to_string(), json!({}), 5000)
/// 		.await?;
/// # Ok(result)
/// # }
/// ```
pub async fn SendRequest(
	SideCarIdentifier:&str,
	Method:String,
	Parameters:Value,
	TimeoutMilliseconds:u64,
) -> Result<Value, VineError> {
	// Short-circuit when shutdown is in progress - the sidecar may
	// already be SIGKILL'd, and pending requests should fail fast
	// rather than block the shutdown sequence on a 5s tonic timeout.
	if IsShuttingDown() {
		return Err(VineError::ClientNotConnected(SideCarIdentifier.to_string()));
	}
	// Validate method name format
	if Method.is_empty() || Method.len() > 128 {
		return Err(VineError::RPCError(
			"Method name must be between 1 and 128 characters".to_string(),
		));
	}

	let timeout_duration = Duration::from_millis(if TimeoutMilliseconds > 0 {
		TimeoutMilliseconds
	} else {
		Config::DEFAULT_TIMEOUT_MS
	});

	// Validate message size
	let parameter_bytes =
		to_vec(&Parameters).map_err(|e| VineError::RPCError(format!("Failed to serialize parameters: {}", e)))?;
	ValidateMessageSize(&parameter_bytes)?;

	let client = {
		let guard = SIDECAR_CLIENTS.lock();
		guard.get(SideCarIdentifier).cloned()
	};

	if client.is_none() {
		return Err(VineError::ClientNotConnected(SideCarIdentifier.to_string()));
	}

	let mut client = client.unwrap();

	let request_identifier = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap()
		.as_nanos() as u64;
	let method_clone = Method.clone();
	let request = GenericRequest { request_identifier, method:Method, parameter:parameter_bytes };

	let result = timeout(timeout_duration, client.process_mountain_request(request)).await;

	match result {
		Ok(Ok(response)) => {
			UpdateSideCarActivity(SideCarIdentifier);
			dev_log!(
				"grpc",
				"[VineClient] Request sent successfully to sidecar '{}': method='{}'",
				SideCarIdentifier,
				method_clone
			);

			// Get the inner response message
			let inner_response = response.into_inner();

			// Parse response JSON
			let result_bytes = inner_response.result;
			let result_value:Value = from_slice(&result_bytes)
				.map_err(|e| VineError::RPCError(format!("Failed to deserialize response: {}", e)))?;

			// Check for RPC errors in response
			if let Some(error_data) = inner_response.error {
				return Err(VineError::RPCError(format!(
					"RPC error from sidecar: code={}, message={}",
					error_data.code, error_data.message
				)));
			}

			Ok(result_value)
		},
		Ok(Err(status)) => {
			RecordSideCarFailure(SideCarIdentifier);
			return Err(VineError::RPCError(format!("gRPC error: {}", status)));
		},
		Err(_) => {
			RecordSideCarFailure(SideCarIdentifier);
			Err(VineError::RequestTimeout {
				SideCarIdentifier:SideCarIdentifier.to_string(),
				MethodName:method_clone,
				TimeoutMilliseconds:timeout_duration.as_millis() as u64,
			})
		},
	}
}

/// Sends a notification to a sidecar without waiting for a response.
///
/// Note: This does not include a timeout parameter (unlike `SendRequest`).
/// Notifications are sent as fire-and-forget messages.
///
/// ```rust,no_run
/// # use Vine::Client::SendNotification;
/// use serde_json::json;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// SendNotification(
///     "cocoon-main".to_string(),
///     "UpdateTheme".to_string(),
///     json!({"theme": "dark"}),
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn SendNotification(SideCarIdentifier:String, Method:String, Parameters:Value) -> Result<(), VineError> {
	// Short-circuit once shutdown has been initiated. Background tokio
	// tasks (PTY reader, watchers, diagnostics emitter) keep producing
	// notifications until they unwind; if Cocoon has already received
	// `$shutdown` (or been SIGKILL'd) those attempts hit ECONNREFUSED
	// and surface as `[VineClient] Failed to send notification` errors
	// in the tail of every log. Drop them silently.
	if IsShuttingDown() {
		return Ok(());
	}
	// Validate method name format
	if Method.is_empty() || Method.len() > 128 {
		return Err(VineError::RPCError(
			"Method name must be between 1 and 128 characters".to_string(),
		));
	}

	let parameter_bytes = to_vec(&Parameters)?;
	ValidateMessageSize(&parameter_bytes)?;

	let mut client = {
		let guard = SIDECAR_CLIENTS.lock();
		guard.get(&SideCarIdentifier).cloned()
	};

	if let Some(ref mut client) = client {
		// Snapshot for the broadcast publish below - `request` moves
		// the owned `Method` into the protobuf message.
		let MethodForPublish = Method.clone();
		let request = GenericNotification { method:Method, parameter:parameter_bytes };

		match client.send_mountain_notification(request).await {
			Ok(_) => {
				UpdateSideCarActivity(&SideCarIdentifier);
				dev_log!(
					"grpc",
					"[VineClient] Notification sent successfully to sidecar '{}'",
					SideCarIdentifier
				);
				// LAND-PATCH B7-S6: fan out to the broadcast channel
				// so any number of subscribers (Effect-TS fibers, OTel
				// span emitters, future Mist-WS bridge, dev log) can
				// observe the same flow concurrently. Slow subscribers
				// drop oldest; producer never stalls.
				PublishNotification(&SideCarIdentifier, &MethodForPublish, &Parameters);
				Ok(())
			},
			Err(status) => {
				RecordSideCarFailure(&SideCarIdentifier);
				dev_log!(
					"grpc",
					"error: [VineClient] Failed to send notification to sidecar '{}': {}",
					SideCarIdentifier,
					status
				);
				Err(VineError::from(status))
			},
		}
	} else {
		Err(VineError::ClientNotConnected(SideCarIdentifier))
	}
}
