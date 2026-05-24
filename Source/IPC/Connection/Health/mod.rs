pub mod New;
pub mod WithTimeout;
pub mod CheckConnectionHealth;
pub mod PingTimeout;
pub mod SetPingTimeout;

use super::Types::ConnectionHandle;
use crate::dev_log;

/// Connection health checker
/// This structure provides periodic health checking for connections,
/// monitoring response times and overall connection health.
/// ## Health Check Process
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
/// ## Health Criteria
/// A connection is considered healthy if:
/// - Response time < ping_timeout (default 5 seconds)
/// ## Example Usage
/// ```rust,ignore
/// let checker = HealthChecker::new();
/// let mut handle = ConnectionHandle::new();
/// let is_healthy = checker.CheckConnectionHealth(&mut handle).await;
/// ```
pub struct Struct {
	/// Maximum allowed response time for a connection to be considered healthy
	ping_timeout:std::time::Duration,
}
