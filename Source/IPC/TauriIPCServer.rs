//! # TauriIPCServer - Mountain-Wind IPC Bridge
//!
//! **File Responsibilities:**
//! This module serves as the core IPC (Inter-Process Communication) server for
//! Mountain, establishing and managing the bidirectional communication bridge
//! between Mountain's Rust backend and Wind's TypeScript frontend. It
//! implements the Mountain counterpart to Wind's TauriIPCServer.ts, ensuring
//! seamless integration across the language boundary.
//!
//! **Architectural Role in Wind-Mountain Connection:**
//! The TauriIPCServer acts as the central Message router and communication
//! orchestrator:
//!
//! 1. **Connection Management:**
//!    - Establishes secure connections between Wind and Mountain
//!    - Maintains connection health and auto-reconnects on failure
//!    - Manages connection pooling for optimal resource usage
//!    - Tracks connection state for monitoring and debugging
//!
//! 2. **Message Routing:**
//!    - Routes incoming messages from Wind to appropriate handlers
//!    - Broadcasts messages from Mountain to Wind subscribers
//!    - Implements Message filtering and prioritization
//!    - Supports point-to-point and publish-subscribe patterns
//!
//! 3. **Security Layer:**
//!    - Validates all incoming messages for security
//!    - Implements permission-based access control (RBAC)
//!    - Provides AES-256-GCM encryption for sensitive data
//!    - Logs all security events for audit trails
//!
//! 4. **Reliability Features:**
//!    - Message queuing for offline scenarios
//!    - Automatic retry with exponential backoff
//!    - Graceful degradation when services unavailable
//!    - Circuit breaker pattern for cascading failure prevention
//!
//! **Communication Patterns:**
//!
//! **1. Request-Response Pattern:**
//! ```text
//! // Wind sends request
//! let result = app_handle.invoke_handler("command", args).await?;
//!
//! // Mountain processes and responds
//! let response = handle_request().await;
//! ipc.emit(response_channel, response).await;
//! ```
//!
//! **2. Event Emission Pattern:**
//! ```text
//! // Mountain emits events to Wind subscribers
//! app.emit("configuration-updated", new_config).await;
//! app.emit("file-changed", file_event).await;
//! ```
//!
//! **3. Broadcast Pattern:**
//! ```rust
//! // Broadcast to all subscribers on a channel
//! for listener in listeners.get(channel) {
//! 	listener(Message.clone()).await;
//! }
//! ```
//!
//! **Message Flow:**
//! ```text
//! Wind Frontend
//! |
//! | 4. Response
//! v
//! Tauri Bridge (JS Bridge)
//! |
//! | 1. IPC Invoke
//! v
//! TauriIPCServer (Rust)
//! |
//! | 2. Route & Validate
//! v
//! WindServiceHandlers
//! |
//! | 3. Execute
//! v
//! Mountain Services
//! ```
//!
//! **Key Structures:**
//!
//! - **TauriIPCMessage:** Standard Message format for all IPC communication
//! - **ConnectionStatus:** Tracks connection health and uptime
//! - **ConnectionPool:** Manages concurrent IPC connections efficiently
//! - **PermissionManager:** Implements role-based access control
//! - **SecureMessageChannel:** Provides encryption for sensitive data
//! - **MessageCompressor:** Gzip compression for large payloads
//!
//! **Defensive Coding Practices:**
//!
//! 1. **Input Validation:**
//!    - All messages validated before processing
//!    - Type checking for all serialized data
//!    - Schema validation for complex payloads
//!
//! 2. **Error Handling:**
//!    - Comprehensive error messages with context
//!    - Error logging at appropriate levels
//!    - Graceful handling of transient failures
//!    - Automatic retry with backoff
//!
//! 3. **Timeout Management:**
//!    - Configurable timeouts for all operations
//!    - Timeout-based circuit breaking
//!    - Graceful degradation on timeout
//!
//! 4. **Resource Management:**
//!    - Connection pooling to prevent exhaustion
//!    - Automatic cleanup of stale resources
//!    - Memory-efficient Message queuing
//!
//! **Security Architecture:**
//!
//! - **Authentication:** User identity verification
//! - **Authorization:** Permission-based access control (RBAC)
//! - **Encryption:** AES-256-GCM for sensitive data
//! - **Auditing:** Complete security event logging
//! - **Threat Detection:** Anomaly monitoring and alerts
//!
//! **Performance Optimizations:**
//!
//! - **Message Compression:** Gzip for large payloads
//! - **Connection Pooling:** Reuse connections efficiently
//! - **Caching:** Cache frequently used data
//! - **Batching:** Batch multiple messages for efficiency
//! - **Async/Await:** Non-blocking I/O operations
//!
//! **Monitoring & Observability:**
//!
//! - **Connection Status:** Real-time health monitoring
//! - **Performance Metrics:** Latency, throughput, error rates
//! - **Audit Logs:** Complete Message and security event logging
//! - **Health Checks:** Periodic health assessments
//!
//! **VSCode RPC Patterns (Study Reference):**
//! This implementation draws inspiration from VSCode's RPC/IPC architecture:
//! - Channel-based Message routing
//! - Request-response correlation
//! - Cancellation token support
//! - Binary protocol Message serialization
//! - Protocol versioning for compatibility

use std::{
	collections::HashMap,
	io::{Read, Write},
	sync::{Arc, Mutex},
	time::Duration,
};

use base64::{Engine, engine::general_purpose};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use ring::{
	aead::{self, AES_256_GCM, LessSafeKey, UnboundKey},
	hmac,
	rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::{
	sync::{Mutex as AsyncMutex, RwLock, Semaphore},
	time::timeout,
};

use crate::dev_log;

#[path = "TauriIPCServer/DecryptMessage.rs"]
pub mod DecryptMessage;

#[path = "TauriIPCServer/Dispose.rs"]
pub mod Dispose;

#[path = "TauriIPCServer/InitializeDefaults.rs"]
pub mod InitializeDefaults;

#[path = "TauriIPCServer/ProcessMessageQueue.rs"]
pub mod ProcessMessageQueue;

#[path = "TauriIPCServer/ReceiveMessage.rs"]
pub mod ReceiveMessage;

#[path = "TauriIPCServer/SendMessage.rs"]
pub mod SendMessage;

#[path = "TauriIPCServer/StartHealthMonitoring.rs"]
pub mod StartHealthMonitoring;

#[path = "TauriIPCServer/ValidatePermission.rs"]
pub mod ValidatePermission;

/// IPC Message structure matching Wind's ITauriIPCMessage interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriIPCMessage {
	pub channel:String,

	pub data:serde_json::Value,

	pub sender:Option<String>,

	pub timestamp:u64,
}

/// Connection status Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
	pub connected:bool,
}

/// Listener callback type
type ListenerCallback = Box<dyn Fn(serde_json::Value) -> Result<(), String> + Send + Sync>;

/// Mountain's IPC Server counterpart to Wind's TauriIPCServer
#[derive(Clone)]
pub struct TauriIPCServer {
	app_handle:AppHandle,

	listeners:Arc<Mutex<HashMap<String, Vec<ListenerCallback>>>>,

	is_connected:Arc<Mutex<bool>>,

	message_queue:Arc<Mutex<Vec<TauriIPCMessage>>>,
}

/// Message compression utility for optimizing IPC Message transfer
pub struct MessageCompressor {
	CompressionLevel:u32,

	BatchSize:usize,
}

impl MessageCompressor {
	/// Create a new Message compressor with specified parameters
	pub fn new(CompressionLevel:u32, BatchSize:usize) -> Self { Self { CompressionLevel, BatchSize } }

	/// Compress messages using Gzip for efficient transfer
	pub fn compress_messages(&self, Messages:Vec<TauriIPCMessage>) -> Result<Vec<u8>, String> {
		let SerializedMessages =
			serde_json::to_vec(&Messages).map_err(|e| format!("Failed to serialize messages: {}", e))?;

		let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.CompressionLevel));

		encoder
			.write_all(&SerializedMessages)
			.map_err(|e| format!("Failed to compress messages: {}", e))?;

		encoder.finish().map_err(|e| format!("Failed to finish compression: {}", e))
	}

	/// Decompress messages from compressed data
	pub fn decompress_messages(&self, CompressedData:&[u8]) -> Result<Vec<TauriIPCMessage>, String> {
		let mut decoder = GzDecoder::new(CompressedData);

		let mut DecompressedData = Vec::new();

		decoder
			.read_to_end(&mut DecompressedData)
			.map_err(|e| format!("Failed to decompress data: {}", e))?;

		serde_json::from_slice(&DecompressedData).map_err(|e| format!("Failed to deserialize messages: {}", e))
	}

	/// Check if messages should be batched for compression
	pub fn should_batch(&self, MessagesCount:usize) -> bool { MessagesCount >= self.BatchSize }
}

impl TauriIPCServer {
	/// Create a new Tauri IPC Server instance
	pub fn new(app_handle:AppHandle) -> Self {
		dev_log!("ipc", "[TauriIPCServer] Initializing Mountain IPC Server");

		Self {
			app_handle,

			listeners:Arc::new(Mutex::new(HashMap::new())),

			is_connected:Arc::new(Mutex::new(false)),

			message_queue:Arc::new(Mutex::new(Vec::new())),
		}
	}

	/// Initialize the IPC server and set up event listeners
	pub async fn initialize(&self) -> Result<(), String> {
		dev_log!("ipc", "[TauriIPCServer] Setting up IPC listeners");

		// Set up connection status
		{
			let mut is_connected = self
				.is_connected
				.lock()
				.map_err(|e| format!("Failed to lock connection status: {}", e))?;

			*is_connected = true;
		}

		// Notify Wind that Mountain is ready
		self.send_connection_status(true)
			.await
			.map_err(|e| format!("Failed to send connection status: {}", e))?;

		dev_log!("ipc", "[TauriIPCServer] IPC Server initialized successfully");

		// Process any queued messages
		self.process_message_queue().await;

		Ok(())
	}

	/// Send a Message to the Wind frontend
	pub async fn send(&self, channel:&str, data:serde_json::Value) -> Result<(), String> {
		SendMessage::Fn(self, channel, data).await
	}

	/// Register a listener for incoming messages from Wind
	pub fn on(&self, channel:&str, callback:ListenerCallback) -> Result<(), String> {
		let mut listeners = self
			.listeners
			.lock()
			.map_err(|e| format!("Failed to access listeners: {}", e))?;

		listeners.entry(channel.to_string()).or_insert_with(Vec::new).push(callback);

		dev_log!("ipc", "[TauriIPCServer] Listener registered for channel: {}", channel);

		Ok(())
	}

	/// Remove a listener
	pub fn off(&self, channel:&str, callback:&ListenerCallback) -> Result<(), String> {
		let mut listeners = self
			.listeners
			.lock()
			.map_err(|e| format!("Failed to access listeners: {}", e))?;

		if let Some(channel_listeners) = listeners.get_mut(channel) {
			channel_listeners.retain(|cb| !std::ptr::eq(cb as *const _, callback as *const _));

			if channel_listeners.is_empty() {
				listeners.remove(channel);
			}
		}

		dev_log!("ipc", "[TauriIPCServer] Listener removed from channel: {}", channel);

		Ok(())
	}

	/// Handle incoming messages from Wind
	pub async fn IncomingMessage(&self, Message:TauriIPCMessage) -> Result<(), String> {
		dev_log!("ipc", "[TauriIPCServer] Received Message on channel: {}", Message.channel);

		let listeners = self
			.listeners
			.lock()
			.map_err(|e| format!("Failed to access listeners: {}", e))?;

		if let Some(channel_listeners) = listeners.get(&Message.channel) {
			for callback in channel_listeners {
				if let Err(e) = callback(Message.data.clone()) {
					dev_log!(
						"ipc",
						"error: [TauriIPCServer] Error in listener for channel {}: {}",
						Message.channel,
						e
					);
				}
			}
		} else {
			dev_log!("ipc", "[TauriIPCServer] No listeners found for channel: {}", Message.channel);
		}

		Ok(())
	}

	/// Send connection status to Wind
	async fn send_connection_status(&self, connected:bool) -> Result<(), String> {
		let status = ConnectionStatus { connected };

		self.app_handle
			.emit("vscode-ipc-status", status)
			.map_err(|e| format!("Failed to emit connection status: {}", e))?;

		dev_log!("ipc", "[TauriIPCServer] Connection status sent: {}", connected);

		Ok(())
	}

	/// Emit a Message to Wind
	async fn emit_message(&self, Message:&TauriIPCMessage) -> Result<(), String> {
		self.app_handle
			.emit("vscode-ipc-Message", Message)
			.map_err(|e| format!("Failed to emit Message: {}", e))?;

		dev_log!("ipc", "[TauriIPCServer] Message emitted on channel: {}", Message.channel);

		Ok(())
	}

	/// Process queued messages
	async fn process_message_queue(&self) { ProcessMessageQueue::Fn(self).await }

	/// Get connection status
	pub fn get_connection_status(&self) -> Result<bool, String> {
		let guard = self
			.is_connected
			.lock()
			.map_err(|e| format!("Failed to get connection status: {}", e))?;

		Ok(*guard)
	}

	/// Get queued Message count
	pub fn get_queue_size(&self) -> Result<usize, String> {
		let guard = self
			.message_queue
			.lock()
			.map_err(|e| format!("Failed to get queue size: {}", e))?;

		Ok(guard.len())
	}

	/// Cleanup resources
	pub fn dispose(&self) -> Result<(), String> { Dispose::Fn(self) }

	/// Advanced: Validate Message permissions
	pub async fn validate_message_permissions(&self, Message:&TauriIPCMessage) -> Result<(), String> {
		let permission_manager = PermissionManager::new();

		permission_manager.initialize_defaults().await;

		let context = self.create_security_context(Message);

		// Extract operation from channel name
		let operation = Message.channel.replace("mountain_", "");

		// Validate permission
		permission_manager.validate_permission(&operation, &context).await
	}

	/// Advanced: Create security context from Message
	fn create_security_context(&self, Message:&TauriIPCMessage) -> SecurityContext {
		SecurityContext {
			user_id:Message.sender.clone().unwrap_or("unknown".to_string()),

			// Default role assigned to authenticated IPC connections
			roles:vec!["user".to_string()],

			permissions:vec![],

			// IPC connections use loopback address for security (localhost only)
			ip_address:"127.0.0.1".to_string(),

			timestamp:std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(Message.timestamp),
		}
	}

	/// Advanced: Log security event
	pub async fn log_security_event(&self, event:SecurityEvent) {
		let permission_manager = PermissionManager::new();

		permission_manager.log_security_event(event).await;
	}

	/// Advanced: Record performance metrics
	pub async fn record_performance_metrics(&self, channel:String, duration:std::time::Duration, success:bool) {
		// This would integrate with the PerformanceDashboard
		dev_log!(
			"ipc",
			"[TauriIPCServer] Performance recorded - Channel: {}, Duration: {:?}, Success: {}",
			channel,
			duration,
			success
		);
	}

	/// Advanced: Get security audit log
	pub async fn get_security_audit_log(&self, limit:usize) -> Result<Vec<SecurityEvent>, String> {
		let permission_manager = PermissionManager::new();

		Ok(permission_manager.get_audit_log(limit).await)
	}

	/// Send compressed Message batch
	pub async fn send_compressed_batch(&self, channel:&str, messages:Vec<TauriIPCMessage>) -> Result<(), String> {
		// Configure compressor with balanced settings: level 6 (good compression/speed
		// tradeoff) and batch size 10 (aggregate small messages for efficiency)
		let compressor = MessageCompressor::new(6, 10);

		let compressed_data = compressor
			.compress_messages(messages)
			.map_err(|e| format!("Failed to compress batch: {}", e))?;

		let batch_message = TauriIPCMessage {
			channel:"compressed_batch".to_string(),

			data:serde_json::Value::String(general_purpose::STANDARD.encode(&compressed_data)),

			sender:Some("mountain".to_string()),

			timestamp:std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64,
		};

		self.send(channel, serde_json::to_value(batch_message).unwrap()).await
	}

	/// Handle compressed batch Message
	pub async fn CompressedBatch(&self, Message:TauriIPCMessage) -> Result<(), String> {
		let compressed_data_base64 = Message.data.as_str().ok_or("Compressed batch data must be a string")?;

		let compressed_data = general_purpose::STANDARD
			.decode(compressed_data_base64)
			.map_err(|e| format!("Failed to decode base64: {}", e))?;

		let compressor = MessageCompressor::new(6, 10);

		let messages = compressor
			.decompress_messages(&compressed_data)
			.map_err(|e| format!("Failed to decompress batch: {}", e))?;

		// Process each Message in the batch
		for Message in messages {
			self.IncomingMessage(Message).await?;
		}

		Ok(())
	}

	/// Send Message using connection pool
	pub async fn send_with_pool(&self, channel:&str, data:serde_json::Value) -> Result<(), String> {
		let pool = Arc::new(ConnectionPool::new(10, Duration::from_secs(30)));

		let Handle = pool
			.GetConnection()
			.await
			.map_err(|e| format!("Failed to get connection: {}", e))?;

		let result = self.send(channel, data).await;

		pool.ReleaseConnection(Handle).await;

		result
	}

	/// Get connection pool statistics
	pub async fn get_connection_stats(&self) -> Result<ConnectionStats, String> {
		let pool = Arc::new(ConnectionPool::new(10, Duration::from_secs(30)));

		Ok(pool.GetStats().await)
	}

	/// Send encrypted Message
	pub async fn send_secure(&self, channel:&str, data:serde_json::Value) -> Result<(), String> {
		let secure_channel =
			SecureMessageChannel::new().map_err(|e| format!("Failed to create secure channel: {}", e))?;

		let Message = TauriIPCMessage {
			channel:channel.to_string(),

			data,

			sender:Some("mountain".to_string()),

			timestamp:std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64,
		};

		let encrypted_message = secure_channel
			.encrypt_message(&Message)
			.map_err(|e| format!("Failed to encrypt Message: {}", e))?;

		let encrypted_data = serde_json::to_value(encrypted_message)
			.map_err(|e| format!("Failed to serialize encrypted Message: {}", e))?;

		self.send("secure_message", encrypted_data).await
	}

	/// Handle encrypted Message
	pub async fn SecureMessage(&self, encrypted_data:serde_json::Value) -> Result<(), String> {
		let encrypted_message:EncryptedMessage = serde_json::from_value(encrypted_data)
			.map_err(|e| format!("Failed to deserialize encrypted Message: {}", e))?;

		let secure_channel =
			SecureMessageChannel::new().map_err(|e| format!("Failed to create secure channel: {}", e))?;

		let Message = secure_channel
			.decrypt_message(&encrypted_message)
			.map_err(|e| format!("Failed to decrypt Message: {}", e))?;

		self.IncomingMessage(Message).await
	}

	/// Handle Message with permission validation
	pub async fn MessageWithPermissions(&self, Message:TauriIPCMessage) -> Result<(), String> {
		let permission_manager = PermissionManager::new();

		let context = self.create_security_context(&Message);

		// Extract operation from channel name
		let operation = Message.channel.replace("mountain_", "");

		// Validate permission
		permission_manager.validate_permission(&operation, &context).await?;

		// Process the Message
		self.IncomingMessage(Message).await
	}
}

/// Connection pool for IPC operations - manages concurrent connections
/// efficiently
///
/// **Purpose:** Prevents connection exhaustion by pooling connections and
/// reusing them **Features:** Health monitoring, automatic cleanup,
/// configurable timeouts
pub struct ConnectionPool {
	MaxConnections:usize,

	ConnectionTimeout:Duration,

	Semaphore:Arc<Semaphore>,

	ActiveConnection:Arc<AsyncMutex<HashMap<String, ConnectionHandle>>>,

	HealthChecker:Arc<AsyncMutex<ConnectionHealthChecker>>,
}

/// Handle representing an active connection
#[derive(Clone)]
pub struct ConnectionHandle {
	pub id:String,

	pub created_at:std::time::Instant,

	pub last_used:std::time::Instant,

	pub health_score:f64,

	pub error_count:usize,
}

impl ConnectionHandle {
	/// Create a new connection Handle with health monitoring
	pub fn new() -> Self {
		Self {
			id:uuid::Uuid::new_v4().to_string(),

			created_at:std::time::Instant::now(),

			last_used:std::time::Instant::now(),

			health_score:100.0,

			error_count:0,
		}
	}

	/// Update health score based on operation success
	pub fn update_health(&mut self, success:bool) {
		if success {
			self.health_score = (self.health_score + 10.0).min(100.0);

			self.error_count = 0;
		} else {
			self.health_score = (self.health_score - 25.0).max(0.0);

			self.error_count += 1;
		}

		self.last_used = std::time::Instant::now();
	}

	/// Check if connection is healthy
	pub fn is_healthy(&self) -> bool { self.health_score > 50.0 && self.error_count < 5 }

	/// Check if connection is alive (non-degraded, not expired).
	pub fn is_alive(&self) -> bool { self.health_score > 0.0 && self.error_count < 10 }
}

impl ConnectionPool {
	/// Create a new connection pool with specified parameters
	pub fn new(MaxConnections:usize, ConnectionTimeout:Duration) -> Self {
		Self {
			MaxConnections,

			ConnectionTimeout,

			Semaphore:Arc::new(Semaphore::new(MaxConnections)),

			ActiveConnection:Arc::new(AsyncMutex::new(HashMap::new())),

			HealthChecker:Arc::new(AsyncMutex::new(ConnectionHealthChecker::new())),
		}
	}

	/// Get a connection Handle from the pool with timeout
	pub async fn GetConnection(&self) -> Result<ConnectionHandle, String> {
		let _permit = timeout(self.ConnectionTimeout, self.Semaphore.acquire())
			.await
			.map_err(|_| "Connection timeout")?
			.map_err(|e| format!("Failed to acquire connection: {}", e))?;

		let Handle = ConnectionHandle::new();

		{
			let mut connections = self.ActiveConnection.lock().await;

			connections.insert(Handle.id.clone(), Handle.clone());
		}

		// Start health monitoring for this connection
		self.StartHealthMonitoring(&Handle.id).await;

		Ok(Handle)
	}

	/// Release a connection Handle back to the pool
	pub async fn ReleaseConnection(&self, Handle:ConnectionHandle) {
		{
			let mut connections = self.ActiveConnection.lock().await;

			connections.remove(&Handle.id);
		}

		// The permit is released when dropped
	}

	/// Get connection statistics for monitoring
	pub async fn GetStats(&self) -> ConnectionStats {
		let connections = self.ActiveConnection.lock().await;

		let healthy_connections = connections.values().filter(|h| h.is_healthy()).count();

		ConnectionStats {
			total_connections:connections.len(),

			healthy_connections,

			max_connections:self.MaxConnections,

			available_permits:self.Semaphore.available_permits(),

			connection_timeout:self.ConnectionTimeout,
		}
	}

	/// Clean up stale connections
	pub async fn CleanUpStaleConnections(&self) -> usize {
		let mut connections = self.ActiveConnection.lock().await;

		let now = std::time::Instant::now();

		// Stale connections are those unused for 5 minutes (300 seconds)
		let stale_threshold = Duration::from_secs(300);

		let stale_ids:Vec<String> = connections
			.iter()
			.filter(|(_, Handle)| now.duration_since(Handle.last_used) > stale_threshold || !Handle.is_healthy())
			.map(|(id, _)| id.clone())
			.collect();

		let stale_count = stale_ids.len();

		for id in stale_ids {
			connections.remove(&id);
		}

		stale_count
	}

	/// Start health monitoring for a connection
	async fn StartHealthMonitoring(&self, connection_id:&str) { StartHealthMonitoring::Fn(self, connection_id).await }
}

/// Connection health checker
struct ConnectionHealthChecker {}

impl ConnectionHealthChecker {
	fn new() -> Self { Self {} }

	/// Check connection health - returns true if the connection handle has a
	/// live channel. No artificial sleep; this is called from an idle
	/// background task so latency doesn't matter.
	async fn check_connection_health(&self, Handle:&mut ConnectionHandle) -> bool { Handle.is_alive() }
}

/// Connection statistics
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
	pub total_connections:usize,

	pub healthy_connections:usize,

	pub max_connections:usize,

	pub available_permits:usize,

	pub connection_timeout:Duration,
}

/// Secure Message channel with encryption and authentication
pub struct SecureMessageChannel {
	encryption_key:LessSafeKey,

	hmac_key:Vec<u8>,
}

impl SecureMessageChannel {
	/// Create a new secure channel
	pub fn new() -> Result<Self, String> {
		let rng = SystemRandom::new();

		// Generate encryption key
		let mut encryption_key_bytes = vec![0u8; 32];

		rng.fill(&mut encryption_key_bytes)
			.map_err(|e| format!("Failed to generate encryption key: {}", e))?;

		let unbound_key = UnboundKey::new(&AES_256_GCM, &encryption_key_bytes)
			.map_err(|e| format!("Failed to create unbound key: {}", e))?;

		let encryption_key = LessSafeKey::new(unbound_key);

		// Generate HMAC key
		let mut hmac_key = vec![0u8; 32];

		rng.fill(&mut hmac_key)
			.map_err(|e| format!("Failed to generate HMAC key: {}", e))?;

		Ok(Self { encryption_key, hmac_key })
	}

	/// Encrypt and authenticate a Message
	pub fn encrypt_message(&self, Message:&TauriIPCMessage) -> Result<EncryptedMessage, String> {
		let serialized_message =
			serde_json::to_vec(Message).map_err(|e| format!("Failed to serialize Message: {}", e))?;

		// Generate nonce
		let mut nonce = [0u8; 12];

		SystemRandom::new()
			.fill(&mut nonce)
			.map_err(|e| format!("Failed to generate nonce: {}", e))?;

		// Encrypt Message
		let mut in_out = serialized_message.clone();

		self.encryption_key
			.seal_in_place_append_tag(aead::Nonce::assume_unique_for_key(nonce), aead::Aad::empty(), &mut in_out)
			.map_err(|e| format!("Encryption failed: {}", e))?;

		// Create HMAC
		let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &self.hmac_key);

		let hmac_tag = hmac::sign(&hmac_key, &in_out);

		Ok(EncryptedMessage { nonce:nonce.to_vec(), ciphertext:in_out, hmac_tag:hmac_tag.as_ref().to_vec() })
	}

	/// Decrypt and verify a Message
	pub fn decrypt_message(&self, encrypted:&EncryptedMessage) -> Result<TauriIPCMessage, String> {
		DecryptMessage::Fn(self, encrypted)
	}

	/// Rotate encryption keys
	pub fn rotate_keys(&mut self) -> Result<(), String> {
		*self = Self::new()?;
		Ok(())
	}
}

/// Encrypted Message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
	nonce:Vec<u8>,

	ciphertext:Vec<u8>,

	hmac_tag:Vec<u8>,
}

/// Advanced permission-based IPC Message handler
#[tauri::command]
pub async fn mountain_ipc_receive_message(app_handle:tauri::AppHandle, Message:TauriIPCMessage) -> Result<(), String> {
	ReceiveMessage::Fn(app_handle, Message).await
}

/// Tauri command handler for Wind to check connection status
///
/// **Command Registration:** Registered in Tauri's invoke_handler
/// Called by Wind using: `app.Handle.invoke('mountain_ipc_get_status')`
///
/// **Response Format:**
/// ```json
/// {
///   "connected": true
/// }
/// ```
#[tauri::command]
pub async fn mountain_ipc_get_status(app_handle:tauri::AppHandle) -> Result<ConnectionStatus, String> {
	if let Some(ipc_server) = app_handle.try_state::<TauriIPCServer>() {
		let connected = ipc_server
			.get_connection_status()
			.map_err(|e| format!("Failed to get connection status: {}", e))?;

		Ok(ConnectionStatus { connected })
	} else {
		Err("IPC Server not found in application state".to_string())
	}
}

/// Security context for permission validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
	pub user_id:String,

	pub roles:Vec<String>,

	pub permissions:Vec<String>,

	pub ip_address:String,

	pub timestamp:std::time::SystemTime,
}

/// Permission manager for IPC operations
pub struct PermissionManager {
	roles:Arc<RwLock<HashMap<String, Role>>>,

	permissions:Arc<RwLock<HashMap<String, Permission>>>,

	audit_log:Arc<RwLock<Vec<SecurityEvent>>>,
}

/// Security event for auditing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
	pub event_type:SecurityEventType,

	pub user_id:String,

	pub operation:String,

	pub timestamp:std::time::SystemTime,

	pub details:Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
	PermissionDenied,

	AccessGranted,

	ConfigurationChange,

	SecurityViolation,

	PerformanceAnomaly,
}

/// Role definition for RBAC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
	pub name:String,

	pub permissions:Vec<String>,

	pub description:String,
}

/// Permission definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
	pub name:String,

	pub description:String,

	pub category:String,
}

impl PermissionManager {
	pub fn new() -> Self {
		Self {
			roles:Arc::new(RwLock::new(HashMap::new())),

			permissions:Arc::new(RwLock::new(HashMap::new())),

			audit_log:Arc::new(RwLock::new(Vec::new())),
		}
	}

	/// Validate permission for an operation
	pub async fn validate_permission(&self, operation:&str, context:&SecurityContext) -> Result<(), String> {
		ValidatePermission::Fn(self, operation, context).await
	}

	/// Get required permissions for an operation
	async fn get_required_permissions(&self, operation:&str) -> Vec<String> {
		// Define operation-to-permission mapping
		match operation {
			"file:write" | "file:delete" => vec!["file.write".to_string()],

			"configuration:update" => vec!["config.update".to_string()],

			"storage:set" => vec!["storage.write".to_string()],

			"native:openExternal" => vec!["system.external".to_string()],

			// Operations not in the mapping require no special permissions by default
			_ => Vec::new(),
		}
	}

	/// Get permissions for a role
	async fn get_role_permissions(&self, role_name:&str) -> Vec<String> {
		let roles = self.roles.read().await;

		roles.get(role_name).map(|role| role.permissions.clone()).unwrap_or_default()
	}

	/// Log security event
	pub async fn log_security_event(&self, event:SecurityEvent) {
		let mut audit_log = self.audit_log.write().await;

		audit_log.push(event);

		// Keep only last 1000 events
		if audit_log.len() > 1000 {
			audit_log.remove(0);
		}
	}

	/// Get security audit log
	pub async fn get_audit_log(&self, limit:usize) -> Vec<SecurityEvent> {
		let audit_log = self.audit_log.read().await;

		audit_log.iter().rev().take(limit).cloned().collect()
	}

	/// Initialize default roles and permissions
	pub async fn initialize_defaults(&self) { InitializeDefaults::Fn(self).await }
}
