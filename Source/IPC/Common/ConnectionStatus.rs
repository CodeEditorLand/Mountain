//! # Connection Status Tracking
//!
//! Provides types for tracking IPC connection health and state.
//! Used across all IPC components to monitor connection status.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Represents the current state of an IPC connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Connection is active and healthy
    Connected,
    /// Connection is being established
    Connecting,
    /// Connection is temporarily unavailable
    Disconnected,
    /// Connection has failed and needs recovery
    Failed,
    /// Connection is being closed gracefully
    Closing,
    /// Connection is closed and will not reopen
    Closed,
}

/// Comprehensive connection status tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    /// Current connection state
    pub state: ConnectionState,
    /// When the connection entered its current state
    pub state_since: Instant,
    /// Count of connection attempts
    pub connection_attempts: u32,
    /// Timestamp of last successful connection
    pub last_connected: Option<Instant>,
    /// Timestamp of last disconnection
    pub last_disconnected: Option<Instant>,
    /// Total uptime duration
    pub total_uptime: Duration,
    /// Reason for last disconnection (if any)
    pub last_error: Option<String>,
}

impl ConnectionStatus {
    /// Create a new connection status
    pub fn new() -> Self {
        Self {
            state: ConnectionState::Disconnected,
            state_since: Instant::now(),
            connection_attempts: 0,
            last_connected: None,
            last_disconnected: None,
            total_uptime: Duration::ZERO,
            last_error: None,
        }
    }

    /// Update connection state
    pub fn update_state(&mut self, new_state: ConnectionState, error: Option<String>) {
        if new_state != self.state {
            // Track downtime if disconnecting
            if self.state == ConnectionState::Connected {
                if let Some(connected_since) = self.last_connected {
                    self.total_uptime += connected_since.elapsed();
                }
            }

            // Update timestamps
            match new_state {
                ConnectionState::Connected => {
                    self.last_connected = Some(Instant::now());
                    self.connection_attempts += 1;
                }
                ConnectionState::Disconnected | ConnectionState::Failed => {
                    self.last_disconnected = Some(Instant::now());
                }
                _ => {}
            }

            self.state = new_state;
            self.state_since = Instant::now();
            self.last_error = error;
        }
    }

    /// Check if connection is currently active
    pub fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }

    /// Check if connection is in a healthy state
    pub fn is_healthy(&self) -> bool {
        matches!(
            self.state,
            ConnectionState::Connected | ConnectionState::Connecting
        )
    }

    /// Get the duration the connection has been in its current state
    pub fn current_state_duration(&self) -> Duration {
        self.state_since.elapsed()
    }

    /// Get the duration since last successful connection
    pub fn time_since_last_connection(&self) -> Option<Duration> {
        self.last_connected.map(|t| t.elapsed())
    }
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        Self::new()
    }
}
