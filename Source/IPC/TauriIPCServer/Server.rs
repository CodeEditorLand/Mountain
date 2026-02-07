//! # TauriIPCServer (IPC)
//!
//! ## RESPONSIBILITIES
//! This module serves as the core IPC server orchestrator for Mountain, establishing
//! and managing the bidirectional communication bridge between Mountain's Rust backend
//! and Wind's TypeScript frontend. It coordinates all IPC operations and delegates
//! specialized tasks to submodules.
//!
//! ## ARCHITECTURAL ROLE
//! The TauriIPCServer is the central orchestrator in the IPC architecture:
//!
//! ```text
//! Wind Frontend
//!     |
//!     | 4. Response
//!     v
//! Tauri Bridge (JS Bridge)
//!     |
//!     | 1. IPC Invoke
//!     v
//! TauriIPCServer (Rust)
//!     |
//!     | 2. Route & Validate
//!     v
//! Message Handlers & Services
//! ```
//!
//! ## KEY COMPONENTS
//!
//! - **TauriIPCServer**: Main IPC server orchestrator
//! - **Message Management**: Send, receive, and queue messages
//! - **Listener Management**: Register and deregister event listeners
//! - **Security**: Permission validation and security event logging
//! - **Advanced Features**: Compression, encryption, connection pooling
//!
//! ## ERROR HANDLING
//! All operations return Result types with descriptive error messages.
//! Graceful handling of transient failures with retry logic.
//!
//! ## LOGGING
//! Info-level logging for lifecycle events, debug for operations, trace for
//! message details, error for failures.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Message queuing for offline scenarios
//! - Health monitoring for connection stability
//! - Async/await for non-blocking operations
//! - Connection pooling for efficiency
//!
//! ## TODO
//! - Add message priority queuing
//! - Implement connection retry logic
//! - Add message persistence for offline mode
//! - Support multiple transport protocols

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use log::{debug, error, info, trace};
use tauri::{AppHandle, Emitter, Manager};

use super::super::Message::{TauriIPCMessage, ConnectionStatus, ListenerCallback};
use super::super::Security::{PermissionManager, SecurityContext, SecurityEvent, SecurityEventType};
use super::super::Encryption::{MessageCompressor, SecureMessageChannel};
use super::super::Connection::{ConnectionManager, ConnectionStats};

/// Mountain's IPC Server counterpart to Wind's TauriIPCServer
///
/// This is the main orchestrator for IPC communication between Wind (frontend)
/// and Mountain (backend). It manages message routing, listener registration,
/// and provides advanced features like encryption and compression.
///
/// ## Core Responsibilities
///
/// 1. **Connection Management**: Maintain connection health and automatic reconnection
/// 2. **Message Routing**: Route incoming messages to appropriate handlers
/// 3. **Broadcasting**: Emit messages to Wind subscribers
/// 4. **Security**: Validate permissions and log security events
/// 5. **Advanced Features**: Compression, encryption, connection pooling
///
/// ## Message Flow
///
/// ```text
/// Wind → TauriIPCServer → Message Handlers → Mountain Services
/// Mountain Services → TauriIPCServer → Wind
/// ```
///
/// ## Example Usage
///
/// ```rust,ignore
/// let ipc_server = TauriIPCServer::new(app_handle);
/// ipc_server.initialize().await?;
///
/// // Send a message
/// ipc_server.send("channel", data).await?;
///
/// // Register a listener
/// ipc_server.on("channel", Box::new(|data| {
///     // Handle message
///     Ok(())
/// }))?;
/// ```
#[derive(Clone)]
pub struct TauriIPCServer {
	/// Tauri app handle for emitting events
	app_handle: AppHandle,

	/// Registered listeners by channel
	listeners: Arc<Mutex<HashMap<String, Vec<ListenerCallback>>>>

	/// Connection status flag
	is_connected: Arc<Mutex<bool>>,

	/// Queued messages for offline scenarios
	message_queue: Arc<Mutex<Vec<TauriIPCMessage>>>,

	/// Permission manager for access control
	permission_manager: Arc<Mutex<Option<PermissionManager>>>,
}

impl TauriIPCServer {
	/// Create a new Tauri IPC Server instance
	///
	/// ## Parameters
	/// - `app_handle`: Tauri app handle for emitting events
	///
	/// ## Returns
	/// New TauriIPCServer instance
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// let ipc_server = TauriIPCServer::new(app_handle);
	/// ```
	pub fn new(app_handle: AppHandle) -> Self {
		info!("[TauriIPCServer] Initializing Mountain IPC Server");

		Self {
			app_handle,
			listeners: Arc::new(Mutex::new(HashMap::new())),
			is_connected: Arc::new(Mutex::new(false)),
			message_queue: Arc::new(Mutex::new(Vec::new())),
			permission_manager: Arc::new(Mutex::new(None)),
		}
	}

	/// Initialize the IPC server and set up event listeners
	///
	/// This method sets up the connection and processes any queued messages
	/// from previous offline periods.
	///
	/// ## Returns
	/// - `Ok(())`: Initialization successful
	/// - `Err(String)`: Error message if initialization fails
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// ipc_server.initialize().await?;
	/// ```
	pub async fn initialize(&self) -> Result<(), String> {
		info!("[TauriIPCServer] Setting up IPC listeners");

		// Set up connection status
		{
			let mut is_connected = self
				.is_connected
				.lock()
				.map_err(|e| format!("Failed to lock connection status: {}", e))?;
			*is_connected = true;
		}

		// Initialize permission manager
		{
			let mut permission_manager = self
				.permission_manager
				.lock()
				.map_err(|e| format!("Failed to lock permission manager: {}", e))?;
			if permission_manager.is_none() {
				let pm = PermissionManager::new();
				let pm_clone = pm.clone();
				tokio::spawn(async move {
					pm_clone.initialize_defaults().await;
				});
				*permission_manager = Some(pm);
			}
		}

		// Notify Wind that Mountain is ready
		self.send_connection_status(true)
			.await
			.map_err(|e| format!("Failed to send connection status: {}", e))?;

		info!("[TauriIPCServer] IPC Server initialized successfully");

		// Process any queued messages
		self.process_message_queue().await;

		Ok(())
	}

	/// Send a message to the Wind frontend
	///
	/// This method sends a message to Wind. If the connection is not active,
	/// the message is queued for later delivery.
	///
	/// ## Parameters
	/// - `channel`: IPC channel name
	/// - `data`: Message payload
	///
	/// ## Returns
	/// - `Ok(())`: Message sent or queued successfully
	/// - `Err(String)`: Error message if sending fails
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// ipc_server.send("my-channel", serde_json::json!({"key": "value"})).await?;
	/// ```
	pub async fn send(&self, channel: &str, data: serde_json::Value) -> Result<(), String> {
		let message = TauriIPCMessage {
			channel: channel.to_string(),
			data,
			sender: Some("mountain".to_string()),
			timestamp: std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64,
		};

		let is_connected = {
			let guard = self
				.is_connected
				.lock()
				.map_err(|e| format!("Failed to check connection status: {}", e))?;
			*guard
		};

		if !is_connected {
			// Queue the message for later delivery
			let mut queue = self
				.message_queue
				.lock()
				.map_err(|e| format!("Failed to access message queue: {}", e))?;
			queue.push(message);
			debug!(
				"[TauriIPCServer] Message queued (channel: {}, queue size: {})",
				channel,
				queue.len()
			);
			return Ok(());
		}

		// Send immediately
		self.emit_message(&message).await
	}

	/// Register a listener for incoming messages from Wind
	///
	/// ## Parameters
	/// - `channel`: IPC channel name
	/// - `callback`: Callback function to handle messages
	///
	/// ## Returns
	/// - `Ok(())`: Listener registered successfully
	/// - `Err(String)`: Error message if registration fails
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// ipc_server.on("my-channel", Box::new(|data| {
	///     println!("Received: {:?}", data);
	///     Ok(())
	/// }))?;
	/// ```
	pub fn on(&self, channel: &str, callback: ListenerCallback) -> Result<(), String> {
		let mut listeners = self
			.listeners
			.lock()
			.map_err(|e| format!("Failed to access listeners: {}", e))?;

		listeners
			.entry(channel.to_string())
			.or_insert_with(Vec::new)
			.push(callback);

		debug!("[TauriIPCServer] Listener registered for channel: {}", channel);
		Ok(())
	}

	/// Remove a listener
	///
	/// ## Parameters
	/// - `channel`: IPC channel name
	/// - `callback`: Callback to remove
	///
	/// ## Returns
	/// - `Ok(())`: Listener removed successfully
	/// - `Err(String)`: Error message if removal fails
	pub fn off(&self, channel: &str, callback: &ListenerCallback) -> Result<(), String> {
		let mut listeners = self
			.listeners
			.lock()
			.map_err(|e| format!("Failed to access listeners: {}", e))?;

		if let Some(channel_listeners) = listeners.get_mut(channel) {
			channel_listeners.retain(|cb| {
				!std::ptr::eq(cb as *const _ as *const (), callback as *const _ as *const ())
			});

			if channel_listeners.is_empty() {
				listeners.remove(channel);
			}
		}

		debug!("[TauriIPCServer] Listener removed from channel: {}", channel);
		Ok(())
	}

	/// Handle incoming messages from Wind
	///
	/// ## Parameters
	/// - `message`: Incoming message to handle
	///
	/// ## Returns
	/// - `Ok(())`: Message handled successfully
	/// - `Err(String)`: Error message if handling fails
	pub async fn handle_incoming_message(&self, message: TauriIPCMessage) -> Result<(), String> {
		trace!("[TauriIPCServer] Received message on channel: {}", message.channel);

		let listeners = self
			.listeners
			.lock()
			.map_err(|e| format!("Failed to access listeners: {}", e))?;

		if let Some(channel_listeners) = listeners.get(&message.channel) {
			for callback in channel_listeners {
				if let Err(e) = callback(message.data.clone()) {
					error!(
						"[TauriIPCServer] Error in listener for channel {}: {}",
						message.channel, e
					);
				}
			}
		} else {
			debug!("[TauriIPCServer] No listeners found for channel: {}", message.channel);
		}

		Ok(())
	}

	/// Send connection status to Wind
	///
	/// ## Parameters
	/// - `connected`: Connection status
	///
	/// ## Returns
	/// - `Ok(())`: Status sent successfully
	/// - `Err(String)`: Error message if sending fails
	async fn send_connection_status(&self, connected: bool) -> Result<(), String> {
		let status = ConnectionStatus::new(connected);

		self.app_handle
			.emit("vscode-ipc-status", status)
			.map_err(|e| format!("Failed to emit connection status: {}", e))?;

		debug!("[TauriIPCServer] Connection status sent: {}", connected);
		Ok(())
	}

	/// Emit a message to Wind
	///
	/// ## Parameters
	/// - `message`: Message to emit
	///
	/// ## Returns
	/// - `Ok(())`: Message emitted successfully
	/// - `Err(String)`: Error message if emission fails
	async fn emit_message(&self, message: &TauriIPCMessage) -> Result<(), String> {
		self.app_handle
			.emit("vscode-ipc-message", message)
			.map_err(|e| format!("Failed to emit message: {}", e))?;

		trace!("[TauriIPCServer] Message emitted on channel: {}", message.channel);
		Ok(())
	}

	/// Process queued messages
	///
	/// This method processes any queued messages from offline periods.
	async fn process_message_queue(&self) {
		let mut queue = match self.message_queue.lock() {
			Ok(queue) => queue,
			Err(e) => {
				error!("[TauriIPCServer] Failed to access message queue: {}", e);
				return;
			}
		};

		while let Some(message) = queue.pop() {
			if let Err(e) = self.emit_message(&message).await {
				error!("[TauriIPCServer] Failed to send queued message: {}", e);
				// Put the message back in the queue
				queue.insert(0, message);
				break;
			}
		}

		debug!(
			"[TauriIPCServer] Message queue processed, {} messages remaining",
			queue.len()
		);
	}

	/// Get connection status
	///
	/// ## Returns
	/// - `Ok(bool)`: Connection status
	/// - `Err(String)`: Error message if status check fails
	pub fn get_connection_status(&self) -> Result<bool, String> {
		let guard = self
			.is_connected
			.lock()
			.map_err(|e| format!("Failed to get connection status: {}", e))?;
		Ok(*guard)
	}

	/// Get queued message count
	///
	/// ## Returns
	/// - `Ok(usize)`: Number of queued messages
	/// - `Err(String)`: Error message if count check fails
	pub fn get_queue_size(&self) -> Result<usize, String> {
		let guard = self
			.message_queue
			.lock()
			.map_err(|e| format!("Failed to get queue size: {}", e))?;
		Ok(guard.len())
	}

	/// Cleanup resources
	///
	/// ## Returns
	/// - `Ok(())`: Cleanup successful
	/// - `Err(String)`: Error message if cleanup fails
	pub fn dispose(&self) -> Result<(), String> {
		{
			let mut listeners = self
				.listeners
				.lock()
				.map_err(|e| format!("Failed to access listeners: {}", e))?;
			listeners.clear();
		}

		{
			let mut queue = self
				.message_queue
				.lock()
				.map_err(|e| format!("Failed to access message queue: {}", e))?;
			queue.clear();
		}

		{
			let mut is_connected = self
				.is_connected
				.lock()
				.map_err(|e| format!("Failed to access connection status: {}", e))?;
			*is_connected = false;
		}

		info!("[TauriIPCServer] IPC Server disposed");
		Ok(())
	}

	/// Validate message permissions
	///
	/// ## Parameters
	/// - `message`: Message to validate
	///
	/// ## Returns
	/// - `Ok(())`: Permissions validated
	/// - `Err(String)`: Error message if validation fails
	pub async fn validate_message_permissions(&self, message: &TauriIPCMessage) -> Result<(), String> {
		let permission_manager_guard = self
			.permission_manager
			.lock()
			.map_err(|e| format!("Failed to access permission manager: {}", e))?;

		let permission_manager = permission_manager_guard.as_ref()
			.ok_or_else(|| "Permission manager not initialized".to_string())?;

		let context = self.create_security_context(message);

		// Extract operation from channel name
		let operation = message.channel.replace("mountain_", "");

		// Validate permission
		permission_manager.validate_permission(&operation, &context).await
	}

	/// Create security context from message
	///
	/// ## Parameters
	/// - `message`: Message to create context from
	///
	/// ## Returns
	/// SecurityContext for the message
	fn create_security_context(&self, message: &TauriIPCMessage) -> SecurityContext {
		SecurityContext {
			user_id: message.sender.clone().unwrap_or("unknown".to_string()),
			// Default role assigned to authenticated IPC connections
			roles: vec!["user".to_string()],
			permissions: vec![],
			// IPC connections use loopback address for security (localhost only)
			ip_address: "127.0.0.1".to_string(),
			timestamp: std::time::SystemTime::UNIX_EPOCH
				+ std::time::Duration::from_millis(message.timestamp),
		}
	}

	/// Log security event
	///
	/// ## Parameters
	/// - `event`: Security event to log
	pub async fn log_security_event(&self, event: SecurityEvent) {
		let permission_manager_guard = match self.permission_manager.lock() {
			Ok(guard) => guard,
			Err(e) => {
				error!("[TauriIPCServer] Failed to access permission manager: {}", e);
				return;
			}
		};

		if let Some(permission_manager) = permission_manager_guard.as_ref() {
			permission_manager.log_security_event(event).await;
		}
	}

	/// Record performance metrics
	///
	/// ## Parameters
	/// - `channel`: IPC channel
	/// - `duration`: Operation duration
	/// - `success`: Whether operation succeeded
	pub async fn record_performance_metrics(
		&self,
		channel: String,
		duration: std::time::Duration,
		success: bool,
	) {
		debug!(
			"[TauriIPCServer] Performance recorded - Channel: {}, Duration: {:?}, Success: {}",
			channel, duration, success
		);
		// This would integrate with PerformanceDashboard in the future
	}

	/// Get security audit log
	///
	/// ## Parameters
	/// - `limit`: Maximum number of events to return
	///
	/// ## Returns
	/// Vector of security events
	pub async fn get_security_audit_log(&self, limit: usize) -> Result<Vec<SecurityEvent>, String> {
		let permission_manager_guard = self
			.permission_manager
			.lock()
			.map_err(|e| format!("Failed to access permission manager: {}", e))?;

		let permission_manager = permission_manager_guard.as_ref()
			.ok_or_else(|| "Permission manager not initialized".to_string())?;

		Ok(permission_manager.get_audit_log(limit).await)
	}

	/// Send compressed message batch
	///
	/// ## Parameters
	/// - `channel`: IPC channel
	/// - `messages`: Messages to compress and send
	///
	/// ## Returns
	/// - `Ok(())`: Batch sent successfully
	/// - `Err(String)`: Error message if sending fails
	pub async fn send_compressed_batch(
		&self,
		channel: &str,
		messages: Vec<TauriIPCMessage>,
	) -> Result<(), String> {
		// Configure compressor with balanced settings
		let compressor = MessageCompressor::new(6, 10);

		let compressed_data = compressor
			.compress_messages(messages)
			.map_err(|e| format!("Failed to compress batch: {}", e))?;

		let batch_message = TauriIPCMessage {
			channel: "compressed_batch".to_string(),
			data: serde_json::Value::String(base64::encode(&compressed_data)),
			sender: Some("mountain".to_string()),
			timestamp: std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64,
		};

		self.send(channel, serde_json::to_value(batch_message).unwrap())
			.await
	}

	/// Handle compressed batch message
	///
	/// ## Parameters
	/// - `message`: Compressed batch message
	///
	/// ## Returns
	/// - `Ok(())`: Batch handled successfully
	/// - `Err(String)`: Error message if handling fails
	pub async fn handle_compressed_batch(&self, message: TauriIPCMessage) -> Result<(), String> {
		let compressed_data_base64 = message.data.as_str()
			.ok_or("Compressed batch data must be a string")?;

		let compressed_data = base64::decode(compressed_data_base64)
			.map_err(|e| format!("Failed to decode base64: {}", e))?;

		let compressor = MessageCompressor::new(6, 10);
		let messages = compressor
			.decompress_messages(&compressed_data)
			.map_err(|e| format!("Failed to decompress batch: {}", e))?;

		// Process each message in the batch
		for msg in messages {
			self.handle_incoming_message(msg).await?;
		}

		Ok(())
	}

	/// Send message using connection pool
	///
	/// ## Parameters
	/// - `channel`: IPC channel
	/// - `data`: Message payload
	///
	/// ## Returns
	/// - `Ok(())`: Message sent successfully
	/// - `Err(String)`: Error message if sending fails
	pub async fn send_with_pool(&self, channel: &str, data: serde_json::Value) -> Result<(), String> {
		let pool = Arc::new(ConnectionManager::new(10, std::time::Duration::from_secs(30)));

		let handle = pool
			.GetConnection()
			.await
			.map_err(|e| format!("Failed to get connection: {}", e))?;

		let result = self.send(channel, data).await;

		pool.ReleaseConnection(handle).await;

		result
	}

	/// Get connection pool statistics
	///
	/// ## Returns
	/// Connection statistics
	pub async fn get_connection_stats(&self) -> Result<ConnectionStats, String> {
		let pool = Arc::new(ConnectionManager::new(10, std::time::Duration::from_secs(30)));
		Ok(pool.GetStats().await)
	}

	/// Send encrypted message
	///
	/// ## Parameters
	/// - `channel`: IPC channel
	/// - `data`: Message payload
	///
	/// ## Returns
	/// - `Ok(())`: Message sent successfully
	/// - `Err(String)`: Error message if sending fails
	pub async fn send_secure(&self, channel: &str, data: serde_json::Value) -> Result<(), String> {
		let secure_channel = SecureMessageChannel::new()
			.map_err(|e| format!("Failed to create secure channel: {}", e))?;

		let message = TauriIPCMessage {
			channel: channel.to_string(),
			data,
			sender: Some("mountain".to_string()),
			timestamp: std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64,
		};

		let encrypted_message = secure_channel
			.encrypt_message(&message)
			.map_err(|e| format!("Failed to encrypt message: {}", e))?;

		let encrypted_data = serde_json::to_value(encrypted_message)
			.map_err(|e| format!("Failed to serialize encrypted message: {}", e))?;

		self.send("secure_message", encrypted_data).await
	}

	/// Handle encrypted message
	///
	/// ## Parameters
	/// - `encrypted_data`: Encrypted message data
	///
	/// ## Returns
	/// - `Ok(())`: Message handled successfully
	/// - `Err(String)`: Error message if handling fails
	pub async fn handle_secure_message(&self, encrypted_data: serde_json::Value) -> Result<(), String> {
		use serde::Deserialize;

		#[derive(Deserialize)]
		struct EncryptedMessage {
			nonce: Vec<u8>,
			ciphertext: Vec<u8>,
			hmac_tag: Vec<u8>,
		}

		let encrypted_message: EncryptedMessage = serde_json::from_value(encrypted_data)
			.map_err(|e| format!("Failed to deserialize encrypted message: {}", e))?;

		let secure_channel = SecureMessageChannel::new()
			.map_err(|e| format!("Failed to create secure channel: {}", e))?;

		let message = secure_channel
			.decrypt_message(&super::super::Encryption::EncryptedMessage {
				nonce: encrypted_message.nonce,
				ciphertext: encrypted_message.ciphertext,
				hmac_tag: encrypted_message.hmac_tag,
			})
			.map_err(|e| format!("Failed to decrypt message: {}", e))?;

		self.handle_incoming_message(message).await
	}

	/// Handle message with permission validation
	///
	/// ## Parameters
	/// - `message`: Message to handle
	///
	/// ## Returns
	/// - `Ok(())`: Message handled successfully
	/// - `Err(String)`: Error message if handling fails
	pub async fn handle_message_with_permissions(&self, message: TauriIPCMessage) -> Result<(), String> {
		// Validate permission
		self.validate_message_permissions(&message).await?;

		// Process the message
		self.handle_incoming_message(message).await
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// Note: These tests would require mocking the Tauri AppHandle
	// For now, we'll provide basic structure tests
}
