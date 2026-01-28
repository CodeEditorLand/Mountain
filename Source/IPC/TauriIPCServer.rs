//! # TauriIPCServer
//! 
//! Mountain counterpart to Wind's TauriIPCServer.ts
//! Provides bidirectional IPC communication between Mountain (Rust backend) and Wind (TypeScript frontend)
//! Uses Tauri's event system for seamless integration

#![allow(non_snake_case, non_camel_case_types)]

use std::{collections::HashMap, sync::{Arc, Mutex}};
use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

/// IPC message structure matching Wind's ITauriIPCMessage interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriIPCMessage {
    pub channel: String,
    pub data: serde_json::Value,
    pub sender: Option<String>,
    pub timestamp: u64,
}

/// Connection status message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub connected: bool,
}

/// Listener callback type
type ListenerCallback = Box<dyn Fn(serde_json::Value) -> Result<(), String> + Send + Sync>;

/// Mountain's IPC Server counterpart to Wind's TauriIPCServer
pub struct TauriIPCServer {
    app_handle: AppHandle,
    listeners: Arc<Mutex<HashMap<String, Vec<ListenerCallback>>>>,
    is_connected: Arc<Mutex<bool>>,
    message_queue: Arc<Mutex<Vec<TauriIPCMessage>>>,
}

impl TauriIPCServer {
    /// Create a new Tauri IPC Server instance
    pub fn new(app_handle: AppHandle) -> Self {
        info!("[TauriIPCServer] Initializing Mountain IPC Server");
        
        Self {
            app_handle,
            listeners: Arc::new(Mutex::new(HashMap::new())),
            is_connected: Arc::new(Mutex::new(false)),
            message_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Initialize the IPC server and set up event listeners
    pub async fn initialize(&self) -> Result<(), String> {
        info!("[TauriIPCServer] Setting up IPC listeners");
        
        // Set up connection status
        {
            let mut is_connected = self.is_connected.lock()
                .map_err(|e| format!("Failed to lock connection status: {}", e))?;
            *is_connected = true;
        }
        
        // Notify Wind that Mountain is ready
        self.send_connection_status(true).await
            .map_err(|e| format!("Failed to send connection status: {}", e))?;
        
        info!("[TauriIPCServer] IPC Server initialized successfully");
        
        // Process any queued messages
        self.process_message_queue().await;
        
        Ok(())
    }

    /// Send a message to the Wind frontend
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
            let guard = self.is_connected.lock()
                .map_err(|e| format!("Failed to check connection status: {}", e))?;
            *guard
        };
        
        if !is_connected {
            // Queue the message for later delivery
            let mut queue = self.message_queue.lock()
                .map_err(|e| format!("Failed to access message queue: {}", e))?;
            queue.push(message);
            debug!("[TauriIPCServer] Message queued (channel: {}, queue size: {})", channel, queue.len());
            return Ok(());
        }
        
        // Send immediately
        self.emit_message(&message).await
    }

    /// Register a listener for incoming messages from Wind
    pub fn on(&self, channel: &str, callback: ListenerCallback) -> Result<(), String> {
        let mut listeners = self.listeners.lock()
            .map_err(|e| format!("Failed to access listeners: {}", e))?;
        
        listeners.entry(channel.to_string())
            .or_insert_with(Vec::new)
            .push(callback);
        
        debug!("[TauriIPCServer] Listener registered for channel: {}", channel);
        Ok(())
    }

    /// Remove a listener
    pub fn off(&self, channel: &str, callback: &ListenerCallback) -> Result<(), String> {
        let mut listeners = self.listeners.lock()
            .map_err(|e| format!("Failed to access listeners: {}", e))?;
        
        if let Some(channel_listeners) = listeners.get_mut(channel) {
            channel_listeners.retain(|cb| !std::ptr::eq(cb as *const _, callback as *const _));
            
            if channel_listeners.is_empty() {
                listeners.remove(channel);
            }
        }
        
        debug!("[TauriIPCServer] Listener removed from channel: {}", channel);
        Ok(())
    }

    /// Handle incoming messages from Wind
    pub async fn handle_incoming_message(&self, message: TauriIPCMessage) -> Result<(), String> {
        trace!("[TauriIPCServer] Received message on channel: {}", message.channel);
        
        let listeners = self.listeners.lock()
            .map_err(|e| format!("Failed to access listeners: {}", e))?;
        
        if let Some(channel_listeners) = listeners.get(&message.channel) {
            for callback in channel_listeners {
                if let Err(e) = callback(message.data.clone()) {
                    error!("[TauriIPCServer] Error in listener for channel {}: {}", message.channel, e);
                }
            }
        } else {
            debug!("[TauriIPCServer] No listeners found for channel: {}", message.channel);
        }
        
        Ok(())
    }

    /// Send connection status to Wind
    async fn send_connection_status(&self, connected: bool) -> Result<(), String> {
        let status = ConnectionStatus { connected };
        
        self.app_handle.emit("vscode-ipc-status", status)
            .map_err(|e| format!("Failed to emit connection status: {}", e))?;
        
        debug!("[TauriIPCServer] Connection status sent: {}", connected);
        Ok(())
    }

    /// Emit a message to Wind
    async fn emit_message(&self, message: &TauriIPCMessage) -> Result<(), String> {
        self.app_handle.emit("vscode-ipc-message", message)
            .map_err(|e| format!("Failed to emit message: {}", e))?;
        
        trace!("[TauriIPCServer] Message emitted on channel: {}", message.channel);
        Ok(())
    }

    /// Process queued messages
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
        
        debug!("[TauriIPCServer] Message queue processed, {} messages remaining", queue.len());
    }

    /// Get connection status
    pub fn get_connection_status(&self) -> Result<bool, String> {
        let guard = self.is_connected.lock()
            .map_err(|e| format!("Failed to get connection status: {}", e))?;
        Ok(*guard)
    }

    /// Get queued message count
    pub fn get_queue_size(&self) -> Result<usize, String> {
        let guard = self.message_queue.lock()
            .map_err(|e| format!("Failed to get queue size: {}", e))?;
        Ok(guard.len())
    }

    /// Cleanup resources
    pub fn dispose(&self) -> Result<(), String> {
        {
            let mut listeners = self.listeners.lock()
                .map_err(|e| format!("Failed to access listeners: {}", e))?;
            listeners.clear();
        }
        
        {
            let mut queue = self.message_queue.lock()
                .map_err(|e| format!("Failed to access message queue: {}", e))?;
            queue.clear();
        }
        
        {
            let mut is_connected = self.is_connected.lock()
                .map_err(|e| format!("Failed to access connection status: {}", e))?;
            *is_connected = false;
        }
        
        info!("[TauriIPCServer] IPC Server disposed");
        Ok(())
    }
}

/// Tauri command handler for Wind to send messages to Mountain
#[tauri::command]
pub async fn mountain_ipc_receive_message(
    app_handle: tauri::AppHandle,
    message: TauriIPCMessage,
) -> Result<(), String> {
    debug!("[TauriIPCServer] Received IPC message from Wind on channel: {}", message.channel);
    
    // Get the IPC server instance from application state
    if let Some(ipc_server) = app_handle.try_state::<TauriIPCServer>() {
        ipc_server.handle_incoming_message(message).await
    } else {
        Err("IPC Server not found in application state".to_string())
    }
}

/// Tauri command handler for Wind to check connection status
#[tauri::command]
pub async fn mountain_ipc_get_status(
    app_handle: tauri::AppHandle,
) -> Result<ConnectionStatus, String> {
    if let Some(ipc_server) = app_handle.try_state::<TauriIPCServer>() {
        let connected = ipc_server.get_connection_status()
            .map_err(|e| format!("Failed to get connection status: {}", e))?;
        
        Ok(ConnectionStatus { connected })
    } else {
        Err("IPC Server not found in application state".to_string())
    }
}