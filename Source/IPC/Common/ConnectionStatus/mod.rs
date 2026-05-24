pub mod New;
pub mod UpdateState;
pub mod IsConnected;
pub mod IsHealthy;
pub mod CurrentStateDuration;
pub mod TimeSinceLastConnection;

use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Serialize)]
pub struct Struct {
	/// Current connection state
	pub state:ConnectionState,

	/// When the connection entered its current state (skipped for serialization
	/// as Instant is not serializable)
	#[serde(skip)]
	pub state_since:Instant,

	/// Count of connection attempts
	pub connection_attempts:u32,

	/// Timestamp of last successful connection (skipped for serialization as
	/// Instant is not serializable)
	#[serde(skip)]
	pub last_connected:Option<Instant>,

	/// Timestamp of last disconnection (skipped for serialization as Instant is
	/// not serializable)
	#[serde(skip)]
	pub last_disconnected:Option<Instant>,

	/// Total uptime duration
	pub total_uptime:Duration,

	/// Reason for last disconnection (if any)
	pub last_error:Option<String>,
}
