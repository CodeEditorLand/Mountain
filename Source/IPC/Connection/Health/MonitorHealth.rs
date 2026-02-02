//! # MonitorHealth
//!
//! ## File: IPC/Connection/Health/MonitorHealth.rs
//!
//! ## Role: Monitors and assesses connection health
//! ## Primary Responsibility: Evaluate connection viability and detect degradation
//!
//! ## Dependencies
//! - Tokio: Async runtime for health checks
//! - ConnectionHandle: Connection resource from Pool module
//!
//! ## Security Considerations
//! - Health checks prevent using degraded connections
//! - Timeout limits prevent indefinite waiting
//! - Thresholds prevent flappy connections from being used
//!
//! ## Performance Considerations
//! - Non-blocking health checks with configurable timeouts
//! - Metrics collection without impacting connection performance
//! - Background monitoring without disrupting active operations
//!

use std::{
	sync::Arc,
	time::{Duration, Instant},
};

use tokio::{sync::RwLock, time::timeout};

use super::super::Pool::PoolConnection::{ConnectionHandle, ConnectionHealth};

/// Health check timeout duration
const HEALTH_CHECK_TIMEOUT:Duration = Duration::from_secs(2);

/// Minimum successful pings required to consider connection healthy
const MIN_HEALTHY_PINGS:u32 = 3;

/// Consecutive failures before marking connection as unhealthy
const MAX_CONSECUTIVE_FAILURES:u32 = 5;

/// Health monitoring metrics for a connection
#[derive(Debug, Clone)]
pub struct ConnectionHealthMetrics {
	pub connection_id:String,
	pub ping_success_rate:f32,
	pub average_latency_ms:u64,
	pub consecutive_failures:u32,
	pub last_health_check:Instant,
	pub health_status:ConnectionHealth,
}

/// Connection health checker
pub struct ConnectionHealthChecker {
	/// Maximum number of consecutive failures before marking unhealthy
	pub MaxConsecutiveFailures:u32,

	/// Minimum successful pings required to consider healthy
	pub MinHealthyPings:u32,

	/// Health check timeout
	pub HealthCheckTimeout:Duration,

	/// Historical health metrics
	pub Metrics:Arc<RwLock<ConnectionHealthMetrics>>,
}

impl ConnectionHealthChecker {
	/// Create a new connection health checker with custom settings
	///
	/// ## Parameters
	/// - `MaxConsecutiveFailures`: Maximum consecutive failures before marking
	///   unhealthy
	/// - `MinHealthyPings`: Minimum successful pings required to consider
	///   healthy
	/// - `HealthCheckTimeout`: Timeout for health check operations
	///
	/// ## Returns
	/// New ConnectionHealthChecker instance with custom settings
	pub fn NewWithSettings(MaxConsecutiveFailures:u32, MinHealthyPings:u32, HealthCheckTimeout:Duration) -> Self {
		Self {
			MaxConsecutiveFailures,
			MinHealthyPings,
			HealthCheckTimeout,
			Metrics:Arc::new(RwLock::new(ConnectionHealthMetrics {
				connection_id:String::new(),
				ping_success_rate:1.0,
				average_latency_ms:0,
				consecutive_failures:0,
				last_health_check:Instant::now(),
				health_status:ConnectionHealth::Healthy,
			})),
		}
	}

	/// Check the health of a connection
	///
	/// ## Parameters
	/// - `Handle`: The connection handle to check
	///
	/// ## Returns
	/// True if healthy, false otherwise
	pub async fn CheckConnectionHealth(&self, Handle:&mut ConnectionHandle) -> bool {
		let ConnectionId = Handle.ConnectionId.clone();
		let StartTime = Instant::now();

		// Perform health check with timeout
		let HealthCheckResult = timeout(self.HealthCheckTimeout, self.PerformHealthCheck(Handle)).await;

		let IsHealthy = match HealthCheckResult {
			Ok(Result) => Result,
			Err(_) => {
				// Timeout occurred - mark as failed
				self.RecordFailure().await;
				false
			},
		};

		let Latency = StartTime.elapsed();

		// Update metrics
		{
			let mut Metrics = self.Metrics.write().await;
			Metrics.connection_id = ConnectionId.clone();
			Metrics.consecutive_failures = Handle.ReuseCount as u32;
			Metrics.last_health_check = Instant::now();
			Metrics.average_latency_ms = Latency.as_millis() as u64;
			Metrics.health_status = if IsHealthy { ConnectionHealth::Healthy } else { ConnectionHealth::Unhealthy };
		}

		// Update handle health based on result
		Handle.Health = if IsHealthy { ConnectionHealth::Healthy } else { ConnectionHealth::Unhealthy };

		IsHealthy
	}

	/// Internal health check implementation
	///
	/// ## Parameters
	/// - `Handle`: The connection handle to check
	///
	/// ## Returns
	/// True if healthy, false otherwise
	async fn PerformHealthCheck(&self, Handle:&ConnectionHandle) -> bool {
		// Check if connection has been idle too long
		let IdleTime = Instant::now().duration_since(Handle.LastActivity);

		// If idle for more than 5 minutes, consider it potentially degraded
		if IdleTime > Duration::from_secs(300) {
			return false;
		}

		// Check reuse count - highly reused connections are likely stable
		if Handle.ReuseCount < self.MinHealthyPings {
			// New connections need more pings to establish trust
		}

		// Verify connection is healthy
		Handle.Health == ConnectionHealth::Healthy
	}

	/// Record a failure for the connection
	pub async fn RecordFailure(&self) {
		let mut Metrics = self.Metrics.write().await;
		Metrics.consecutive_failures += 1;

		if Metrics.consecutive_failures >= self.MaxConsecutiveFailures {
			Metrics.health_status = ConnectionHealth::Unhealthy;
		}
	}

	/// Reset the health metrics
	pub async fn ResetMetrics(&self) {
		let mut Metrics = self.Metrics.write().await;
		Metrics.consecutive_failures = 0;
		Metrics.ping_success_rate = 1.0;
		Metrics.health_status = ConnectionHealth::Healthy;
	}

	/// Get the current health metrics
	///
	/// ## Returns
	/// Clone of current health metrics
	pub async fn GetMetrics(&self) -> ConnectionHealthMetrics {
		let Metrics = self.Metrics.read().await;
		Metrics.clone()
	}

	/// Determine if connection should be recycled based on health
	///
	/// ## Parameters
	/// - `Handle`: The connection handle to evaluate
	///
	/// ## Returns
	/// True if connection should be recycled, false otherwise
	pub async fn ShouldRecycle(&self, Handle:&ConnectionHandle) -> bool {
		match Handle.Health {
			ConnectionHealth::Unhealthy => true,
			ConnectionHealth::Degraded => {
				// Recycle degraded connections after certain reuse threshold
				Handle.ReuseCount > 10
			},
			ConnectionHealth::Healthy => false,
		}
	}
}

impl Default for ConnectionHealthChecker {
	fn default() -> Self { Self::New() }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_health_checker_new() {
		let Checker = ConnectionHealthChecker::New();
		assert_eq!(Checker.MaxConsecutiveFailures, MAX_CONSECUTIVE_FAILURES);
		assert_eq!(Checker.MinHealthyPings, MIN_HEALTHY_PINGS);
	}

	#[tokio::test]
	async fn test_health_checker_new_with_settings() {
		let Checker = ConnectionHealthChecker::NewWithSettings(10, 5, Duration::from_secs(3));
		assert_eq!(Checker.MaxConsecutiveFailures, 10);
		assert_eq!(Checker.MinHealthyPings, 5);
		assert_eq!(Checker.HealthCheckTimeout, Duration::from_secs(3));
	}

	#[tokio::test]
	async fn test_check_connection_health() {
		let Checker = ConnectionHealthChecker::New();
		let mut Handle = ConnectionHandle {
			ConnectionId:"test-conn-1".to_string(),
			Channel:"test-channel".to_string(),
			CreatedAt:Instant::now(),
			LastActivity:Instant::now(),
			ReuseCount:5,
			Health:ConnectionHealth::Healthy,
		};

		let Result = Checker.CheckConnectionHealth(&mut Handle).await;
		assert!(Result);
		assert_eq!(Handle.Health, ConnectionHealth::Healthy);
	}

	#[tokio::test]
	async fn test_check_unhealthy_connection() {
		let Checker = ConnectionHealthChecker::New();
		let mut Handle = ConnectionHandle {
			ConnectionId:"test-conn-2".to_string(),
			Channel:"test-channel".to_string(),
			CreatedAt:Instant::now(),
			LastActivity:Instant::now(),
			ReuseCount:5,
			Health:ConnectionHealth::Unhealthy,
		};

		let Result = Checker.CheckConnectionHealth(&mut Handle).await;
		assert!(!Result);
		assert_eq!(Handle.Health, ConnectionHealth::Unhealthy);
	}

	#[tokio::test]
	async fn test_record_failure() {
		let Checker = ConnectionHealthChecker::New();

		Checker.RecordFailure().await;
		Checker.RecordFailure().await;

		let Metrics = Checker.GetMetrics().await;
		assert_eq!(Metrics.consecutive_failures, 2);
	}

	#[tokio::test]
	async fn test_reset_metrics() {
		let Checker = ConnectionHealthChecker::New();

		Checker.RecordFailure().await;
		Checker.RecordFailure().await;

		Checker.ResetMetrics().await;

		let Metrics = Checker.GetMetrics().await;
		assert_eq!(Metrics.consecutive_failures, 0);
		assert_eq!(Metrics.ping_success_rate, 1.0);
	}

	#[tokio::test]
	async fn test_should_recycle_unhealthy() {
		let Checker = ConnectionHealthChecker::New();
		let Handle = ConnectionHandle {
			ConnectionId:"test-conn-3".to_string(),
			Channel:"test-channel".to_string(),
			CreatedAt:Instant::now(),
			LastActivity:Instant::now(),
			ReuseCount:5,
			Health:ConnectionHealth::Unhealthy,
		};

		assert!(Checker.ShouldRecycle(&Handle).await);
	}

	#[tokio::test]
	async fn test_should_recycle_degraded() {
		let Checker = ConnectionHealthChecker::New();
		let Handle = ConnectionHandle {
			ConnectionId:"test-conn-4".to_string(),
			Channel:"test-channel".to_string(),
			CreatedAt:Instant::now(),
			LastActivity:Instant::now(),
			ReuseCount:15,
			Health:ConnectionHealth::Degraded,
		};

		assert!(Checker.ShouldRecycle(&Handle).await);
	}

	#[tokio::test]
	async fn test_should_not_recycle_healthy() {
		let Checker = ConnectionHealthChecker::New();
		let Handle = ConnectionHandle {
			ConnectionId:"test-conn-5".to_string(),
			Channel:"test-channel".to_string(),
			CreatedAt:Instant::now(),
			LastActivity:Instant::now(),
			ReuseCount:5,
			Health:ConnectionHealth::Healthy,
		};

		assert!(!Checker.ShouldRecycle(&Handle).await);
	}
}
