//! # Connection Types (IPC Connection)
//!
//! ## RESPONSIBILITIES
//! This module defines the core data structures for connection management in
//! the IPC layer, including connection handles, statistics, and status
//! tracking.
//!
//! ## ARCHITECTURAL ROLE
//! This module provides the type definitions used throughout the connection
//! management subsystem, ensuring type safety and consistency.
//!
//! ## KEY COMPONENTS
//!
//! - **ConnectionHandle**: Represents an active connection with health tracking
//! - **ConnectionStats**: Statistics about the connection pool
//! - **ConnectionStatus**: Connection health status
//!
//! ## ERROR HANDLING
//! N/A - This is a data definition module.
//!
//! ## LOGGING
//! N/A - Status changes are logged by the ConnectionManager.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - ConnectionHandle uses health scoring for efficient monitoring
//! - Stats are calculated on-demand to avoid overhead
//! - Simple structures minimize memory footprint
//!
//! ## TODO
//! - Add connection metadata (protocol, endpoint)
//! - Implement connection duration tracking
//! - Add connection quality metrics
//! - Support connection tagging for categorization

use serde::{Deserialize, Serialize};

/// Connection status
///
/// This enum represents the current state of an IPC connection, allowing
/// the system to track and report connection health.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {

	/// Connection is active and healthy
	Connected,

	/// Connection is disconnected
	Disconnected,

	/// Connection is degraded (intermittent issues)
	Degraded,

	/// Connection has failed
	Failed,
}

impl ConnectionStatus {

	/// Check if connection is active
	pub fn is_connected(&self) -> bool { matches!(self, ConnectionStatus::Connected) }

	/// Check if connection has issues
	pub fn has_issues(&self) -> bool { matches!(self, ConnectionStatus::Degraded | ConnectionStatus::Failed) }

	/// Get human-readable description
	pub fn description(&self) -> &'static str {
		match self {
			ConnectionStatus::Connected => "Connected and healthy",

			ConnectionStatus::Disconnected => "Disconnected",

			ConnectionStatus::Degraded => "Degraded - experiencing issues",

			ConnectionStatus::Failed => "Failed - connection lost",
		}
	}

	/// Get the status level (0=failed, 1=degraded, 2=disconnected, 3=connected)
	pub fn level(&self) -> u8 {
		match self {
			ConnectionStatus::Failed => 0,

			ConnectionStatus::Degraded => 1,

			ConnectionStatus::Disconnected => 2,

			ConnectionStatus::Connected => 3,
		}
	}
}

impl From<bool> for ConnectionStatus {

	fn from(connected:bool) -> Self {
		if connected {
			ConnectionStatus::Connected
		} else {
			ConnectionStatus::Disconnected
		}
	}
}

/// Handle representing an active connection
///
/// This structure tracks the state and health of an individual connection
/// in the connection pool.
///
/// ## Health Scoring
///
/// The health score ranges from 0.0 to 100.0:
/// - 100.0: Perfect health
/// - 75.0-99.9: Good health
/// - 50.0-74.9: Degraded health
/// - 0.0-49.9: Poor health
///
/// Health is updated based on operation success/failure:
/// - Success: +10 points (max 100)
/// - Failure: -25 points (min 0)
///
/// ## Example Usage
///
/// ```rust,ignore
/// let mut handle = ConnectionHandle::new();
///
/// // Update health based on operation success
/// handle.update_health(true); // Success
/// handle.update_health(false); // Failure
///
/// // Check if connection is healthy
/// if handle.is_healthy() {
///     // Use the connection
/// }
/// ```
#[derive(Clone, Serialize, Deserialize)]
pub struct ConnectionHandle {

	/// Unique connection identifier (UUID)
	pub id:String,

	/// When the connection was created (as SystemTime for serialization)
	pub created_at:std::time::SystemTime,

	/// When the connection was last used (as SystemTime for serialization)
	pub last_used:std::time::SystemTime,

	/// Health score (0.0 to 100.0)
	pub health_score:f64,

	/// Number of consecutive errors
	pub error_count:usize,
}

impl ConnectionHandle {

	/// Create a new connection handle with health monitoring
	pub fn new() -> Self {
		let now = std::time::SystemTime::now();

		Self {
			id:uuid::Uuid::new_v4().to_string(),

			created_at:now,

			last_used:now,

			health_score:100.0,

			error_count:0,
		}
	}

	/// Update health score based on operation success
	///
	/// ## Parameters
	/// - `success`: Whether the operation succeeded
	///
	/// ## Behavior
	/// - Success: +10 points (capped at 100), reset error count
	/// - Failure: -25 points (floored at 0), increment error count
	pub fn update_health(&mut self, success:bool) {
		if success {
			self.health_score = (self.health_score + 10.0).min(100.0);

			self.error_count = 0;
		} else {
			self.health_score = (self.health_score - 25.0).max(0.0);

			self.error_count += 1;
		}

		self.last_used = std::time::SystemTime::now();
	}

	/// Check if connection is healthy
	///
	/// A connection is considered healthy if:
	/// - Health score > 50.0
	/// - Error count < 5
	///
	/// ## Returns
	/// - `true`: Connection is healthy
	/// - `false`: Connection is unhealthy
	pub fn is_healthy(&self) -> bool { self.health_score > 50.0 && self.error_count < 5 }

	/// Get connection age in seconds
	pub fn age_seconds(&self) -> u64 {
		self.created_at
			.duration_since(std::time::UNIX_EPOCH)
			.map(|d| d.as_secs())
			.unwrap_or(0)
	}

	/// Get time since last use in seconds
	pub fn idle_seconds(&self) -> u64 {
		self.last_used
			.duration_since(std::time::UNIX_EPOCH)
			.map(|d| d.as_secs())
			.unwrap_or(0)
	}

	/// Get connection status
	pub fn status(&self) -> ConnectionStatus {
		if self.is_healthy() {
			ConnectionStatus::Connected
		} else if self.health_score > 25.0 {
			ConnectionStatus::Degraded
		} else {
			ConnectionStatus::Failed
		}
	}

	/// Manually update the last used time
	pub fn touch(&mut self) { self.last_used = std::time::SystemTime::now(); }

	/// Reset health score to perfect
	pub fn reset_health(&mut self) {
		self.health_score = 100.0;

		self.error_count = 0;

		self.last_used = std::time::SystemTime::now();
	}
}

/// Helper trait to get duration since UNIX epoch for SystemTime
trait SystemTimeExt {

	/// Get the duration since UNIX epoch in seconds
	fn duration_since_epoch_secs(&self) -> Result<u64, std::time::SystemTimeError>;
}

impl SystemTimeExt for std::time::SystemTime {

	fn duration_since_epoch_secs(&self) -> Result<u64, std::time::SystemTimeError> {
		self.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs())
	}
}

impl std::fmt::Debug for ConnectionHandle {

	fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let created_age = self
			.created_at
			.duration_since(std::time::UNIX_EPOCH)
			.map(|d| d.as_secs())
			.unwrap_or(0);

		let last_used_age = self
			.last_used
			.duration_since(std::time::UNIX_EPOCH)
			.map(|d| d.as_secs())
			.unwrap_or(0);

		f.debug_struct("ConnectionHandle")
			.field("id", &self.id)
			.field("created_at_age_seconds", &created_age)
			.field("last_used_age_seconds", &last_used_age)
			.field("health_score", &self.health_score)
			.field("error_count", &self.error_count)
			.field("status", &self.status())
			.finish()
	}
}

/// Connection statistics for monitoring
///
/// This structure provides aggregate statistics about the connection pool,
/// useful for monitoring and debugging.
///
/// ## Example Usage
///
/// ```rust,ignore
/// let stats = pool.GetStats().await;
///
/// println!("Total connections: {}", stats.total_connections);
/// println!("Healthy: {}", stats.healthy_connections);
/// println!("Available: {}", stats.available_permits);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {

	/// Total number of active connections
	pub total_connections:usize,

	/// Number of healthy connections
	pub healthy_connections:usize,

	/// Maximum number of connections allowed
	pub max_connections:usize,

	/// Number of available connection permits
	pub available_permits:usize,

	/// Connection timeout duration
	pub connection_timeout:std::time::Duration,
}

impl ConnectionStats {

	/// Calculate connection pool utilization percentage
	///
	/// ## Returns
	/// Percentage of connections in use (0.0 to 100.0)
	pub fn utilization(&self) -> f64 {
		if self.max_connections == 0 {
			return 0.0;
		}

		let used = self.max_connections - self.available_permits;

		(used as f64 / self.max_connections as f64) * 100.0
	}

	/// Calculate health percentage
	///
	/// ## Returns
	/// Percentage of connections that are healthy (0.0 to 100.0)
	pub fn health_percentage(&self) -> f64 {
		if self.total_connections == 0 {
			return 100.0;
		}

		(self.healthy_connections as f64 / self.total_connections as f64) * 100.0
	}

	/// Check if pool is under stress
	///
	/// Pool is under stress if:
	/// - Utilization > 80%
	/// - Health percentage < 70%
	///
	/// ## Returns
	/// - `true`: Pool is under stress
	/// - `false`: Pool is healthy
	pub fn is_under_stress(&self) -> bool { self.utilization() > 80.0 || self.health_percentage() < 70.0 }

	/// Get a human-readable status summary
	pub fn summary(&self) -> String {
		format!(
			"Connections: {}/{} ({}%), Healthy: {}%, Utilization: {}%",

			self.total_connections,

			self.max_connections,

			self.health_percentage(),

			self.health_percentage(),

			self.utilization()
		)
	}
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_connection_status_from_bool() {
		assert!(matches!(ConnectionStatus::from(true), ConnectionStatus::Connected));

		assert!(matches!(ConnectionStatus::from(false), ConnectionStatus::Disconnected));
	}

	#[test]
	fn test_connection_status_description() {
		assert_eq!(ConnectionStatus::Connected.description(), "Connected and healthy");

		assert_eq!(ConnectionStatus::Disconnected.description(), "Disconnected");

		assert_eq!(ConnectionStatus::Degraded.description(), "Degraded - experiencing issues");

		assert_eq!(ConnectionStatus::Failed.description(), "Failed - connection lost");
	}

	#[test]
	fn test_connection_status_level() {
		assert_eq!(ConnectionStatus::Failed.level(), 0);

		assert_eq!(ConnectionStatus::Degraded.level(), 1);

		assert_eq!(ConnectionStatus::Disconnected.level(), 2);

		assert_eq!(ConnectionStatus::Connected.level(), 3);
	}

	#[test]
	fn test_connection_handle_creation() {
		let handle = ConnectionHandle::new();

		assert!(!handle.id.is_empty());

		assert_eq!(handle.health_score, 100.0);

		assert_eq!(handle.error_count, 0);

		assert!(handle.is_healthy());
	}

	#[test]
	fn test_connection_handle_health_update_success() {
		let mut handle = ConnectionHandle::new();

		// Initially healthy
		assert_eq!(handle.health_score, 100.0);

		assert!(handle.is_healthy());

		// Simulate success (already at 100, should stay at 100)
		handle.update_health(true);

		assert_eq!(handle.health_score, 100.0);

		assert_eq!(handle.error_count, 0);

		// Simulate failure
		handle.update_health(false);

		assert_eq!(handle.health_score, 75.0);

		assert_eq!(handle.error_count, 1);

		assert!(handle.is_healthy());

		// More failures
		handle.update_health(false);

		assert_eq!(handle.health_score, 50.0);

		assert_eq!(handle.error_count, 2);

		assert!(!handle.is_healthy()); // Health <= 50

		// Recovery
		handle.update_health(true);

		assert_eq!(handle.health_score, 60.0);

		assert_eq!(handle.error_count, 0);

		assert!(handle.is_healthy());
	}

	#[test]
	fn test_connection_handle_health_boundaries() {
		let mut handle = ConnectionHandle::new();

		// Test upper bound (100)
		for _ in 0..20 {
			handle.update_health(true);
		}

		assert_eq!(handle.health_score, 100.0);

		// Reset
		handle.health_score = 50.0;

		// Test lower bound (0)
		for _ in 0..10 {
			handle.update_health(false);
		}

		assert_eq!(handle.health_score, 0.0);
	}

	#[test]
	fn test_connection_handle_is_healthy() {
		let mut handle = ConnectionHandle::new();

		assert!(handle.is_healthy());

		// Make unhealthy via health score
		handle.health_score = 50.0;

		handle.error_count = 0;

		assert!(!handle.is_healthy()); // Health <= 50

		// Make unhealthy via error count
		handle.health_score = 60.0;

		handle.error_count = 5;

		assert!(!handle.is_healthy()); // Errors >= 5
	}

	#[test]
	fn test_connection_handle_status() {
		let mut handle = ConnectionHandle::new();

		assert!(matches!(handle.status(), ConnectionStatus::Connected));

		handle.health_score = 75.0;

		assert!(matches!(handle.status(), ConnectionStatus::Connected));

		handle.health_score = 50.0;

		assert!(matches!(handle.status(), ConnectionStatus::Degraded));

		handle.health_score = 25.0;

		assert!(matches!(handle.status(), ConnectionStatus::Failed));
	}

	#[test]
	fn test_connection_handle_reset() {
		let mut handle = ConnectionHandle::new();

		// Degrade the connection
		for _ in 0..3 {
			handle.update_health(false);
		}

		assert!(handle.health_score < 100.0);

		// Reset
		handle.reset_health();

		assert_eq!(handle.health_score, 100.0);

		assert_eq!(handle.error_count, 0);
	}

	#[test]
	fn test_connection_stats_utilization() {
		let stats = ConnectionStats {
			total_connections:50,

			healthy_connections:45,

			max_connections:100,

			available_permits:50,

			connection_timeout:std::time::Duration::from_secs(30),
		};

		// 50 used out of 100 = 50%
		assert_eq!(stats.utilization(), 50.0);
	}

	#[test]
	fn test_connection_stats_health_percentage() {
		let stats = ConnectionStats {
			total_connections:50,

			healthy_connections:45,

			max_connections:100,

			available_permits:50,

			connection_timeout:std::time::Duration::from_secs(30),
		};

		// 45 healthy out of 50 total = 90%
		assert_eq!(stats.health_percentage(), 90.0);
	}

	#[test]
	fn test_connection_stats_is_under_stress() {
		let mut stats = ConnectionStats {
			total_connections:50,

			healthy_connections:45,

			max_connections:100,

			available_permits:50,

			connection_timeout:std::time::Duration::from_secs(30),
		};

		// Not under stress
		assert!(!stats.is_under_stress());

		// High utilization (90%)
		stats.available_permits = 10;

		assert!(stats.is_under_stress());

		// Low health percentage
		stats.available_permits = 50;

		stats.healthy_connections = 30; // 60%

		assert!(stats.is_under_stress());
	}

	#[test]
	fn test_connection_stats_empty_pool() {
		let stats = ConnectionStats {
			total_connections:0,

			healthy_connections:0,

			max_connections:100,

			available_permits:100,

			connection_timeout:std::time::Duration::from_secs(30),
		};

		assert_eq!(stats.utilization(), 0.0);

		assert_eq!(stats.health_percentage(), 100.0); // Empty pool is healthy

		assert!(!stats.is_under_stress());
	}
}
