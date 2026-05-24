pub mod New;
pub mod Initialize;
pub mod Send;
pub mod On;
pub mod Off;
pub mod IncomingMessage;
pub mod GetConnectionStatus;
pub mod GetQueueSize;
pub mod Dispose;
pub mod ValidateMessagePermissions;
pub mod LogSecurityEvent;
pub mod RecordPerformanceMetrics;
pub mod GetSecurityAuditLog;
pub mod SendCompressedBatch;
pub mod CompressedBatch;
pub mod SendWithPool;
pub mod GetConnectionStats;
pub mod SendSecure;
pub mod SecureMessage;
pub mod MessageWithPermissions;

use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};

use tauri::{AppHandle, Emitter, Manager};

use super::super::{
	Connection::{ConnectionManager, ConnectionStats},
	Encryption::{SecureMessageChannel, Struct},
	Message::{ConnectionStatus, ListenerCallback, TauriIPCMessage},
	Security::PermissionManager::{
		Manager::Struct as PermissionManager,
		SecurityContext::Struct as SecurityContext,
		SecurityEvent::Struct as SecurityEvent,
		SecurityEventType::Enum as SecurityEventType,
	},
};
use crate::dev_log;

/// Mountain's IPC Server counterpart to Wind's TauriIPCServer
/// This is the main orchestrator for IPC communication between Wind (frontend)
/// and Mountain (backend). It manages Message routing, listener registration,
/// and provides advanced features like encryption and compression.
/// ## Core Responsibilities
/// 1. **Connection Management**: Maintain connection health and automatic
///    reconnection
/// 2. **Message Routing**: Route incoming messages to appropriate handlers
/// 3. **Broadcasting**: Emit messages to Wind subscribers
/// 4. **Security**: Validate permissions and log security events
/// 5. **Advanced Features**: Compression, encryption, connection pooling
/// ## Message Flow
/// ```text
/// Wind → TauriIPCServer → Message Handlers → Mountain Services
/// Mountain Services → TauriIPCServer → Wind
/// ```
/// ## Example Usage
/// ```rust,ignore
/// let ipc_server = TauriIPCServer::new(app_handle);
/// ipc_server.Initialize().await?;
/// // Send a Message
/// ipc_server.send("channel", data).await?;
/// // Register a listener
/// ipc_server.On("channel", Box::new(|data| {
///     // Handle Message
///     Ok(())
/// }))?;
/// ```
#[derive(Clone)]
pub struct TauriIPCServer {
	/// Tauri app Handle for emitting events
	app_handle:AppHandle,

	/// Registered listeners by channel
	listeners:Arc<Mutex<HashMap<String, Vec<ListenerCallback>>>>,

	/// Connection status flag
	is_connected:Arc<Mutex<bool>>,

	/// Queued messages for offline scenarios
	message_queue:Arc<Mutex<Vec<TauriIPCMessage>>>,

	/// Permission manager for access control
	permission_manager:Arc<Mutex<Option<PermissionManager>>>,
}

#[derive(Debug, Clone)]
pub struct Struct;
