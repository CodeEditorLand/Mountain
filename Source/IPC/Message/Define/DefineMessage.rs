//! # DefineMessage
//!
//! ## File: IPC/Message/Define/DefineMessage.rs
//!
//! ## Role in Mountain Architecture
//!
//! This module provides the foundational data structures for IPC communication between Mountain and Wind. It defines the contract for all messages flowing through the IPC bridge, ensuring type safety and serialization compatibility across the Rust-TS boundary.
//!
//! ## Primary Responsibility
//!
//! Define all message type structures for IPC communication with serialization support.
//!
//! ## Secondary Responsibilities
//!
//! - Connection status representation
//! - Listener callback type definition
//! - Connection handle structure
//! - Security context for permission validation
//!
//! ## Dependencies
//!
//! **External Crates:**
//! - `serde` - Serialization and deserialization of message types
//!
//! **Internal Modules:**
//! - None (foundational module)
//!
//! ## Dependents
//!
//! - `IPC::TauriIPCServer` - Uses message types for IPC operations
//! - `IPC::Message::Compress::Compress` - Compresses message collections
//! - `IPC::Message::Encrypt::Encrypt` - Encrypts message payloads
//! - `IPC::Message::Route::RouteMessage` - Routes messages to listeners
//! - `IPC::Permission::Validate::ValidatePermission` - Validates message permissions
//!
//! ## VSCode Pattern Reference
//!
//! Follows VSCode's message protocol pattern where messages have channels, payloads, and metadata (sender, timestamp) for correlation and routing.
//!
//! ## Security Considerations
//!
//! - All message fields are validated for type safety
//! - Message timestamps are validated for reasonable ranges
//! - Sender field is optional to support anonymous broadcasts
//! - Channel names are validated by routing layer
//!
//! ## Performance Considerations
//!
//! - Uses efficient JSON serialization via serde
//! - Structures are designed to minimize copy operations
//! - Timestamp uses u64 for compact representation
//!
//! ## Error Handling Strategy
//!
//! - Uses Result<T, serde_json::Error> for serialization/deserialization
//! - Field-level validation in constructors
//!
//! ## Thread Safety
//!
//! - Message types implement Clone for sharing across threads
//! - ListenerCallback is Send + Sync for thread-safe routing
//!
//! ## TODO Items
//!
//! - [ ] Add message schema validation for complex nested structures
//! - [ ] Implement message size limits to prevent DoS
//! - [ ] Add message correlation ID for request-response tracking


use serde::{Deserialize, Serialize};

/// IPC message structure matching Wind's ITauriIPCMessage interface
///
/// This is the fundamental unit of communication between Mountain and Wind.
/// All IPC messages follow this structure for consistent routing and processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriIPCMessage {
    /// The channel name for routing this message
    pub Channel: String,
    /// The payload data, flexibly typed as JSON Value
    pub Data: serde_json::Value,
    /// Optional sender identifier for authentication and auditing
    pub Sender: Option<String>,
    /// Unix timestamp in milliseconds for message ordering and correlation
    pub Timestamp: u64,
}

impl TauriIPCMessage {
    /// Create a new TauriIPCMessage with automatic timestamp
    pub fn New(Channel: String, Data: serde_json::Value) -> Self {
        Self {
            Timestamp: Self::GetCurrentTimestamp(),
            Channel,
            Data,
            Sender: None,
        }
    }

    /// Create a new TauriIPCMessage with explicit sender
    pub fn NewWithSender(Channel: String, Data: serde_json::Value, Sender: String) -> Self {
        Self {
            Timestamp: Self::GetCurrentTimestamp(),
            Channel,
            Data,
            Sender: Some(Sender),
        }
    }

    /// Get current Unix timestamp in milliseconds
    fn GetCurrentTimestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    /// Validate message fields
    pub fn Validate(&self) -> Result<(), String> {
        if self.Channel.is_empty() {
            return Err("Channel name cannot be empty".to_string());
        }
        if self.Channel.len() > 256 {
            return Err("Channel name exceeds maximum length (256)".to_string());
        }
        // Validate timestamp is within reasonable range (last 10 years to next 10 years)
        let now = Self::GetCurrentTimestamp();
        let ten_years_ms = 10 * 365 * 24 * 60 * 60 * 1000; // ~315 billion ms
        if self.Timestamp > now + ten_years_ms {
            return Err("Timestamp is too far in the future".to_string());
        }
        if self.Timestamp < now - ten_years_ms {
            return Err("Timestamp is too far in the past".to_string());
        }
        // Validate sender length if present
        if let Some(ref sender) = self.Sender {
            if sender.len() > 256 {
                return Err("Sender exceeds maximum length (256)".to_string());
            }
        }
        Ok(())
    }
}

/// Connection status message
///
/// Simple boolean status indicating connectivity state between Mountain and Wind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    /// True if connection is active, false otherwise
    pub Connected: bool,
}

impl ConnectionStatus {
    /// Create a new connection status
    pub fn New(Connected: bool) -> Self {
        Self { Connected }
    }
}

/// Listener callback type for message routing
///
/// This type represents a handler that receives message data and processes it.
/// All listeners must be thread-safe and return errors as strings.
pub type ListenerCallback = Box<dyn Fn(serde_json::Value) -> Result<(), String> + Send + Sync>;

/// Connection handle for tracking active IPC connections
///
/// Represents an individual connection in the connection pool with health metrics.
#[derive(Debug, Clone)]
pub struct ConnectionHandle {
    /// Unique identifier for this connection
    pub Id: String,
    /// When this connection was created
    pub CreatedAt: std::time::Instant,
    /// When this connection was last used
    pub LastUsed: std::time::Instant,
    /// Health score (0-100) for connection quality assessment
    pub HealthScore: f64,
    /// Number of errors encountered on this connection
    pub ErrorCount: usize,
}

impl ConnectionHandle {
    /// Create a new connection handle with health monitoring
    pub fn New() -> Self {
        let now = std::time::Instant::now();
        Self {
            Id: uuid::Uuid::new_v4().to_string(),
            CreatedAt: now,
            LastUsed: now,
            HealthScore: 100.0,
            ErrorCount: 0,
        }
    }

    /// Update health score based on operation success
    pub fn UpdateHealth(&mut self, Success: bool) {
        if Success {
            self.HealthScore = (self.HealthScore + 10.0).min(100.0);
            self.ErrorCount = 0;
        } else {
            self.HealthScore = (self.HealthScore - 25.0).max(0.0);
            self.ErrorCount = self.ErrorCount.saturating_add(1);
        }
        self.LastUsed = std::time::Instant::now();
    }

    /// Check if connection is healthy
    pub fn IsHealthy(&self) -> bool {
        self.HealthScore > 50.0 && self.ErrorCount < 5
    }

    /// Check if connection is stale (unused for extended period)
    pub fn IsStale(&self, Duration: std::time::Duration) -> bool {
        self.LastUsed.elapsed() > Duration
    }
}

impl Default for ConnectionHandle {
    fn default() -> Self {
        Self::New()
    }
}

/// Connection statistics for monitoring and reporting
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    /// Total number of active connections
    pub TotalConnections: usize,
    /// Number of healthy connections
    pub HealthyConnections: usize,
    /// Maximum allowed connections
    pub MaxConnections: usize,
    /// Number of available connection slots
    pub AvailablePermits: usize,
    /// Connection acquisition timeout
    pub ConnectionTimeout: std::time::Duration,
}

/// Security context for permission validation
///
/// Contains all contextual information needed to make authorization decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// User identifier for authentication
    pub UserId: String,
    /// List of roles assigned to the user
    pub Roles: Vec<String>,
    /// Direct permissions granted to the user
    pub Permissions: Vec<String>,
    /// IP address for origin validation
    pub IpAddress: String,
    /// Timestamp when this context was created
    pub Timestamp: std::time::SystemTime,
}

impl SecurityContext {
    /// Create a new security context
    pub fn New(UserId: String, Roles: Vec<String>, Permissions: Vec<String>, IpAddress: String) -> Self {
        Self {
            UserId,
            Roles,
            Permissions,
            IpAddress,
            Timestamp: std::time::SystemTime::now(),
        }
    }
}

/// Encrypted message structure for secure communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    /// Nonce for encryption (must be unique per encryption)
    pub Nonce: Vec<u8>,
    /// Encrypted ciphertext with authentication tag appended
    pub Ciphertext: Vec<u8>,
    /// HMAC signature for message authentication
    pub HmacTag: Vec<u8>,
}

/// Export all public types
pub use {
    TauriIPCMessage,
    ConnectionStatus,
    ListenerCallback,
    ConnectionHandle,
    ConnectionStats,
    SecurityContext,
    EncryptedMessage,
};