//! # Health Checker (IPC Connection)
//!
//! ## RESPONSIBILITIES
//! This module provides connection health checking functionality for the
//! IPC layer. It monitors connection health through periodic checks and
//! provides metrics for debugging and monitoring.
//!
//! ## ARCHITECTURAL ROLE
//! This module is part of the monitoring subsystem in the connection management
//! layer, providing health assessments for active connections.
//!
//! ## KEY COMPONENTS
//!
//! - **HealthChecker**: Periodic connection health monitoring
//!
//! ## ERROR HANDLING
// Health checks return boolean results indicating health status.
//
// ## LOGGING
// Debug-level logging for health check results.
//
// ## Performance Considerations
// - Health checks run in background tasks
// - Non-blocking implementation using Tokio
// - Configurable check intervals
// - Timeout-based health assessment
//
// ## TODO
// - Add configurable health check strategies
// - Implement health check customization
// - Add health history tracking
// - Support multiple health check types


use super::Types::ConnectionHandle;
use crate::dev_log;

/// Connection health checker
///
/// This structure provides periodic health checking for connections,
/// monitoring response times and overall connection health.
///
/// ## Health Check Process
///
/// ```text
/// Connection
///     |
///     | 1. Send ping
///     v
/// Measure response time
///     |
///     | 2. Compare to timeout
///     v
/// Health decision (healthy/unhealthy)
/// ```
///
/// ## Health Criteria
///
/// A connection is considered healthy if:
/// - Response time < ping_timeout (default 5 seconds)
///
/// ## Example Usage
///
/// ```rust,ignore
/// let checker = HealthChecker::new();
/// let mut handle = ConnectionHandle::new();
///
/// let is_healthy = checker.check_connection_health(&mut handle).await;
/// ```
pub struct HealthChecker {
	/// Maximum allowed response time for a connection to be considered healthy
	ping_timeout:std::time::Duration,
}

impl HealthChecker {
	/// Create a new health checker with default settings
	///
	/// Default ping timeout is 5 seconds.
	pub fn new() -> Self {
		dev_log!("ipc", "[HealthChecker] Creating health checker with 5s timeout");
		Self { ping_timeout:std::time::Duration::from_secs(5) }
	}

	/// Create a new health checker with custom timeout
	///
	/// ## Parameters
	/// - `ping_timeout`: Maximum allowed response time
	pub fn with_timeout(ping_timeout:std::time::Duration) -> Self {
		dev_log!("ipc", "[HealthChecker] Creating health checker with {:?} timeout", ping_timeout);
		Self { ping_timeout }
	}

	/// Check connection health by sending a ping
	///
	/// This method simulates a health check by measuring response time.
	/// In a production environment, this would send an actual ping message
	/// through the connection.
	///
	/// ## Parameters
	/// - `handle`: Mutable reference to the connection handle to update based
	///   on health
	///
	/// ## Returns
	/// - `true`: Connection is healthy
	/// - `false`: Connection is unhealthy
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// let is_healthy = checker.check_connection_health(&mut handle).await;
	/// ```
	pub async fn check_connection_health(&self, handle:&mut ConnectionHandle) -> bool {
		let start_time = std::time::Instant::now();

		// Simulate network latency (in production, this would be an actual ping)
		// Using a small delay to simulate realistic network conditions
		tokio::time::sleep(std::time::Duration::from_millis(10)).await;

		let response_time = start_time.elapsed();

		// Connection is healthy if response time is within timeout
		let is_healthy = response_time < self.ping_timeout;

		if is_healthy {
			dev_log!("ipc", 
				"[HealthChecker] Connection {} is healthy (response time: {:?})",
				handle.id, response_time
			);
		} else {
			dev_log!("ipc", 
				"[HealthChecker] Connection {} is unhealthy (response time: {:?}, timeout: {:?})",
				handle.id, response_time, self.ping_timeout
			);
		}

		is_healthy
	}

	/// Get the ping timeout
	pub fn ping_timeout(&self) -> std::time::Duration { self.ping_timeout }

	/// Set a new ping timeout
	pub fn set_ping_timeout(&mut self, timeout:std::time::Duration) {
		self.ping_timeout = timeout;
		dev_log!("ipc", "[HealthChecker] Ping timeout updated to {:?}", timeout);
	}
}

impl Default for HealthChecker {
	fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_health_checker_creation() {
		let checker = HealthChecker::new();
		assert_eq!(checker.ping_timeout, std::time::Duration::from_secs(5));
	}

	#[tokio::test]
	async fn test_health_checker_custom_timeout() {
		let timeout = std::time::Duration::from_secs(10);
		let checker = HealthChecker::with_timeout(timeout);
		assert_eq!(checker.ping_timeout, timeout);
	}

	#[tokio::test]
	async fn test_check_connection_health_healthy() {
		let checker = HealthChecker::new();
		let mut handle = ConnectionHandle::new();

		let is_healthy = checker.check_connection_health(&mut handle).await;
		assert!(is_healthy);
	}

	#[tokio::test]
	async fn test_check_connection_health_unhealthy() {
		// Create a checker with very short timeout
		let timeout = std::time::Duration::from_millis(1);
		let checker = HealthChecker::with_timeout(timeout);
		let mut handle = ConnectionHandle::new();

		// The simulated latency (10ms) should exceed the timeout (1ms)
		let is_healthy = checker.check_connection_health(&mut handle).await;
		assert!(!is_healthy);
	}

	#[test]
	fn test_default_health_checker() {
		let checker = HealthChecker::default();
		assert_eq!(checker.ping_timeout, std::time::Duration::from_secs(5));
	}

	#[test]
	fn test_set_ping_timeout() {
		let mut checker = HealthChecker::new();
		assert_eq!(checker.ping_timeout, std::time::Duration::from_secs(5));

		let new_timeout = std::time::Duration::from_secs(15);
		checker.set_ping_timeout(new_timeout);
		assert_eq!(checker.ping_timeout, new_timeout);
	}
}
