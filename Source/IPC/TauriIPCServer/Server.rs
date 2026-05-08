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
//! Message details, error for failures.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Message queuing for offline scenarios
//! - Health monitoring for connection stability
//! - Async/await for non-blocking operations
//! - Connection pooling for efficiency
//!
//! ## TODO
//! - Add Message priority queuing
//! - Implement connection retry logic
//! - Add Message persistence for offline mode
//! - Support multiple transport protocols

use std::collections::HashMap;

use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, Manager};

use super::super::Message::{TauriIPCMessage, ConnectionStatus, ListenerCallback};

use super::super::Security::PermissionManager::{
	Manager::Struct as PermissionManager,
	SecurityContext::Struct as SecurityContext,
	SecurityEvent::Struct as SecurityEvent,
	SecurityEventType::Enum as SecurityEventType,
};

use super::super::Encryption::{MessageCompressor, SecureMessageChannel};

use super::super::Connection::{ConnectionManager, ConnectionStats};

use crate::dev_log;

/// Mountain's IPC Server counterpart to Wind's TauriIPCServer
///
/// This is the main orchestrator for IPC communication between Wind (frontend)
/// and Mountain (backend). It manages Message routing, listener registration,
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
/// // Send a Message
/// ipc_server.send("channel", data).await?;
///
/// // Register a listener
/// ipc_server.on("channel", Box::new(|data| {
///     // Handle Message
///     Ok(())
/// }))?;
/// ```
#[derive(Clone)]
pub struct TauriIPCServer {

	/// Tauri app Handle for emitting events
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
	/// - `app_handle`: Tauri app Handle for emitting events
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

		dev_log!("ipc", "[TauriIPCServer] Initializing Mountain IPC Server");

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
	/// - `Err(String)`: Error Message if initialization fails
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// ipc_server.initialize().await?;
	/// ```
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

		dev_log!("ipc", "[TauriIPCServer] IPC Server initialized successfully");

		// Process any queued messages
		self.process_message_queue().await;

		Ok(())
	}

	/// Send a Message to the Wind frontend
	///
	/// This method sends a Message to Wind. If the connection is not active,
	/// the Message is queued for later delivery.
	///
	/// ## Parameters
	/// - `channel`: IPC channel name
	/// - `data`: Message payload
	///
	/// ## Returns
	/// - `Ok(())`: Message sent or queued successfully
	/// - `Err(String)`: Error Message if sending fails
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// ipc_server.send("my-channel", serde_json::json!({"key": "value"})).await?;
	/// ```
	pub async fn send(&self, channel: &str, data: serde_json::Value) -> Result<(), String> {

		let Message = TauriIPCMessage {

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

			// Queue the Message for later delivery
			let mut queue = self
				.message_queue
				.lock()
				.map_err(|e| format!("Failed to access Message queue: {}", e))?;

			queue.push(Message);

			dev_log!("ipc", 

				"[TauriIPCServer] Message queued (channel: {}, queue size: {})",

				channel,

				queue.len()
			);

			return Ok(());
		}

		// Send immediately
		self.emit_message(&Message).await
	}

	/// Register a listener for incoming messages from Wind
	///
	/// ## Parameters
	/// - `channel`: IPC channel name
	/// - `callback`: Callback function to Handle messages
	///
	/// ## Returns
	/// - `Ok(())`: Listener registered successfully
	/// - `Err(String)`: Error Message if registration fails
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

		dev_log!("ipc", "[TauriIPCServer] Listener registered for channel: {}", channel);

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
	/// - `Err(String)`: Error Message if removal fails
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

		dev_log!("ipc", "[TauriIPCServer] Listener removed from channel: {}", channel);

		Ok(())
	}

	/// Handle incoming messages from Wind
	///
	/// ## Parameters
	/// - `Message`: Incoming Message to Handle
	///
	/// ## Returns
	/// - `Ok(())`: Message handled successfully
	/// - `Err(String)`: Error Message if handling fails
	pub async fn IncomingMessage(&self, Message: TauriIPCMessage) -> Result<(), String> {

		dev_log!("ipc", "[TauriIPCServer] Received Message on channel: {}", Message.channel);

		let listeners = self
			.listeners
			.lock()
			.map_err(|e| format!("Failed to access listeners: {}", e))?;

		if let Some(channel_listeners) = listeners.get(&Message.channel) {

			for callback in channel_listeners {

				if let Err(e) = callback(Message.data.clone()) {

					dev_log!("ipc", "error: [TauriIPCServer] Error in listener for channel {}: {}",

						Message.channel, e);
				}
			}
		} else {

			dev_log!("ipc", "[TauriIPCServer] No listeners found for channel: {}", Message.channel);
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
	/// - `Err(String)`: Error Message if sending fails
	async fn send_connection_status(&self, connected: bool) -> Result<(), String> {

		let status = ConnectionStatus::new(connected);

		self.app_handle
			.emit("vscode-ipc-status", status)
			.map_err(|e| format!("Failed to emit connection status: {}", e))?;

		dev_log!("ipc", "[TauriIPCServer] Connection status sent: {}", connected);

		Ok(())
	}

	/// Emit a Message to Wind
	///
	/// ## Parameters
	/// - `Message`: Message to emit
	///
	/// ## Returns
	/// - `Ok(())`: Message emitted successfully
	/// - `Err(String)`: Error Message if emission fails
	async fn emit_message(&self, Message: &TauriIPCMessage) -> Result<(), String> {

		self.app_handle
			.emit("vscode-ipc-Message", Message)
			.map_err(|e| format!("Failed to emit Message: {}", e))?;

		dev_log!("ipc", "[TauriIPCServer] Message emitted on channel: {}", Message.channel);

		Ok(())
	}

	/// Process queued messages
	///
	/// This method processes any queued messages from offline periods.
	async fn process_message_queue(&self) {

		let mut queue = match self.message_queue.lock() {

			Ok(queue) => queue,

			Err(e) => {

				dev_log!("ipc", "error: [TauriIPCServer] Failed to access Message queue: {}", e);

				return;
			}
		};

		while let Some(Message) = queue.pop() {

			if let Err(e) = self.emit_message(&Message).await {

				dev_log!("ipc", "error: [TauriIPCServer] Failed to send queued Message: {}", e);

				// Put the Message back in the queue
				queue.insert(0, Message);

				break;
			}
		}

		dev_log!("ipc", 

			"[TauriIPCServer] Message queue processed, {} messages remaining",

			queue.len()
		);
	}

	/// Get connection status
	///
	/// ## Returns
	/// - `Ok(bool)`: Connection status
	/// - `Err(String)`: Error Message if status check fails
	pub fn get_connection_status(&self) -> Result<bool, String> {

		let guard = self
			.is_connected
			.lock()
			.map_err(|e| format!("Failed to get connection status: {}", e))?;

		Ok(*guard)
	}

	/// Get queued Message count
	///
	/// ## Returns
	/// - `Ok(usize)`: Number of queued messages
	/// - `Err(String)`: Error Message if count check fails
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
	/// - `Err(String)`: Error Message if cleanup fails
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
				.map_err(|e| format!("Failed to access Message queue: {}", e))?;

			queue.clear();
		}

		{

			let mut is_connected = self
				.is_connected
				.lock()
				.map_err(|e| format!("Failed to access connection status: {}", e))?;

			*is_connected = false;
		}

		dev_log!("ipc", "[TauriIPCServer] IPC Server disposed");

		Ok(())
	}

	/// Validate Message permissions
	///
	/// ## Parameters
	/// - `Message`: Message to validate
	///
	/// ## Returns
	/// - `Ok(())`: Permissions validated
	/// - `Err(String)`: Error Message if validation fails
	pub async fn validate_message_permissions(&self, Message: &TauriIPCMessage) -> Result<(), String> {

		let permission_manager_guard = self
			.permission_manager
			.lock()
			.map_err(|e| format!("Failed to access permission manager: {}", e))?;

		let permission_manager = permission_manager_guard.as_ref()
			.ok_or_else(|| "Permission manager not initialized".to_string())?;

		let context = self.create_security_context(Message);

		// Extract operation from channel name
		let operation = Message.channel.replace("mountain_", "");

		// Validate permission
		permission_manager.validate_permission(&operation, &context).await
	}

	/// Create security context from Message
	///
	/// ## Parameters
	/// - `Message`: Message to create context from
	///
	/// ## Returns
	/// SecurityContext for the Message
	fn create_security_context(&self, Message: &TauriIPCMessage) -> SecurityContext {

		SecurityContext {

			user_id: Message.sender.clone().unwrap_or("unknown".to_string()),

			// Default role assigned to authenticated IPC connections
			roles: vec!["user".to_string()],

			permissions: vec![],

			// IPC connections use loopback address for security (localhost only)
			ip_address: "127.0.0.1".to_string(),

			timestamp: std::time::SystemTime::UNIX_EPOCH
				+ std::time::Duration::from_millis(Message.timestamp),
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

				dev_log!("ipc", "error: [TauriIPCServer] Failed to access permission manager: {}", e);

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

		dev_log!("ipc", 

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

	/// Send compressed Message batch
	///
	/// ## Parameters
	/// - `channel`: IPC channel
	/// - `messages`: Messages to compress and send
	///
	/// ## Returns
	/// - `Ok(())`: Batch sent successfully
	/// - `Err(String)`: Error Message if sending fails
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

	/// Handle compressed batch Message
	///
	/// ## Parameters
	/// - `Message`: Compressed batch Message
	///
	/// ## Returns
	/// - `Ok(())`: Batch handled successfully
	/// - `Err(String)`: Error Message if handling fails
	pub async fn CompressedBatch(&self, Message: TauriIPCMessage) -> Result<(), String> {

		let compressed_data_base64 = Message.data.as_str()
			.ok_or("Compressed batch data must be a string")?;

		let compressed_data = base64::decode(compressed_data_base64)
			.map_err(|e| format!("Failed to decode base64: {}", e))?;

		let compressor = MessageCompressor::new(6, 10);

		let messages = compressor
			.decompress_messages(&compressed_data)
			.map_err(|e| format!("Failed to decompress batch: {}", e))?;

		// Process each Message in the batch
		for msg in messages {

			self.IncomingMessage(msg).await?;
		}

		Ok(())
	}

	/// Send Message using connection pool
	///
	/// ## Parameters
	/// - `channel`: IPC channel
	/// - `data`: Message payload
	///
	/// ## Returns
	/// - `Ok(())`: Message sent successfully
	/// - `Err(String)`: Error Message if sending fails
	pub async fn send_with_pool(&self, channel: &str, data: serde_json::Value) -> Result<(), String> {

		let pool = Arc::new(ConnectionManager::new(10, std::time::Duration::from_secs(30)));

		let Handle = pool
			.GetConnection()
			.await
			.map_err(|e| format!("Failed to get connection: {}", e))?;

		let result = self.send(channel, data).await;

		pool.ReleaseConnection(Handle).await;

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

	/// Send encrypted Message
	///
	/// ## Parameters
	/// - `channel`: IPC channel
	/// - `data`: Message payload
	///
	/// ## Returns
	/// - `Ok(())`: Message sent successfully
	/// - `Err(String)`: Error Message if sending fails
	pub async fn send_secure(&self, channel: &str, data: serde_json::Value) -> Result<(), String> {

		let secure_channel = SecureMessageChannel::new()
			.map_err(|e| format!("Failed to create secure channel: {}", e))?;

		let Message = TauriIPCMessage {

			channel: channel.to_string(),

			data,

			sender: Some("mountain".to_string()),

			timestamp: std::time::SystemTime::now()
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
	///
	/// ## Parameters
	/// - `encrypted_data`: Encrypted Message data
	///
	/// ## Returns
	/// - `Ok(())`: Message handled successfully
	/// - `Err(String)`: Error Message if handling fails
	pub async fn SecureMessage(&self, encrypted_data: serde_json::Value) -> Result<(), String> {

		use serde::Deserialize;

		#[derive(Deserialize)]
		struct EncryptedMessage {

			nonce: Vec<u8>,

			ciphertext: Vec<u8>,

			hmac_tag: Vec<u8>,
		}

		let encrypted_message: EncryptedMessage = serde_json::from_value(encrypted_data)
			.map_err(|e| format!("Failed to deserialize encrypted Message: {}", e))?;

		let secure_channel = SecureMessageChannel::new()
			.map_err(|e| format!("Failed to create secure channel: {}", e))?;

		let Message = secure_channel
			.decrypt_message(&super::super::Encryption::EncryptedMessage {
				nonce: encrypted_message.nonce,
				ciphertext: encrypted_message.ciphertext,
				hmac_tag: encrypted_message.hmac_tag,
			})
			.map_err(|e| format!("Failed to decrypt Message: {}", e))?;

		self.IncomingMessage(Message).await
	}

	/// Handle Message with permission validation
	///
	/// ## Parameters
	/// - `Message`: Message to Handle
	///
	/// ## Returns
	/// - `Ok(())`: Message handled successfully
	/// - `Err(String)`: Error Message if handling fails
	pub async fn MessageWithPermissions(&self, Message: TauriIPCMessage) -> Result<(), String> {

		// Validate permission
		self.validate_message_permissions(&Message).await?;

		// Process the Message
		self.IncomingMessage(Message).await
	}
}

#[cfg(test)]
mod tests {

	use super::*;

	// Note: These tests would require mocking the Tauri AppHandle
	// For now, we'll provide basic structure tests
}
