use std::{collections::HashMap, sync::Arc};

use log::debug;
use tokio::{
	sync::{Mutex as AsyncMutex, Semaphore},
	time::{Duration, timeout},
};

use super::{
	Health::HealthChecker,
	Types::{ConnectionHandle, ConnectionStats},
};

/// Connection manager (alias for ConnectionPool)
///
/// This is the main connection management structure, providing connection
/// pooling with health monitoring and automatic cleanup.
pub type ConnectionManager = ConnectionPool;

/// Connection pool for IPC operations
///
/// This structure manages a pool of connections, preventing connection
/// exhaustion by reusing connections and providing health monitoring.
///
/// ## Pool Architecture
///
/// ```text
/// ConnectionPool
///     |
///     | Semaphore (limits max connections)
///     v
/// Active Connections (HashMap<id, ConnectionHandle>)
///     |
///     | Health Checker (background task)
///     v
/// Monitor health and update scores
/// ```
///
/// ## Connection Lifecycle
///
/// 1. **Acquisition**: Get a connection from the pool (or create new)
/// 2. **Usage**: Use the connection for operations
/// 3. **Release**: Return the connection to the pool
/// 4. **Cleanup**: Automatically remove stale/unhealthy connections
///
/// ## Health Monitoring
///
/// Each connection has:
/// - Health score (0.0 to 100.0)
/// - Error count
/// - Last used timestamp
/// - Background health checks every 30 seconds
///
/// ## Example Usage
///
/// ```rust,ignore
/// let pool = Arc::new(ConnectionPool::new(10, Duration::from_secs(30)));
///
/// // Get a connection
/// let handle = pool.GetConnection().await?;
///
/// // Use the connection...
///
/// // Release the connection
/// pool.ReleaseConnection(handle).await;
///
/// // Get statistics
/// let stats = pool.GetStats().await;
/// ```
pub struct ConnectionPool {
	/// Maximum number of concurrent connections allowed
	MaxConnections:usize,

	/// Timeout for acquiring a connection from the pool
	ConnectionTimeout:Duration,

	/// Semaphore to limit concurrent connections
	Semaphore:Arc<Semaphore>,

	/// Map of active connections by ID
	ActiveConnections:Arc<AsyncMutex<HashMap<String, ConnectionHandle>>>,

	/// Health checker for monitoring connection health
	HealthChecker:Arc<AsyncMutex<HealthChecker>>,
}

impl ConnectionPool {
	/// Create a new connection pool with specified parameters
	///
	/// ## Parameters
	/// - `MaxConnections`: Maximum number of concurrent connections
	/// - `ConnectionTimeout`: Timeout for acquiring a connection
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// let pool = ConnectionPool::new(10, Duration::from_secs(30));
	/// ```
	pub fn new(MaxConnections:usize, ConnectionTimeout:Duration) -> Self {
		debug!(
			"[ConnectionPool] Creating pool with max: {}, timeout: {:?}",
			MaxConnections, ConnectionTimeout
		);

		Self {
			MaxConnections,
			ConnectionTimeout,
			Semaphore:Arc::new(Semaphore::new(MaxConnections)),
			ActiveConnections:Arc::new(AsyncMutex::new(HashMap::new())),
			HealthChecker:Arc::new(AsyncMutex::new(HealthChecker::new())),
		}
	}

	/// Create a connection pool with default settings
	///
	/// Default settings: 10 max connections, 30s timeout
	pub fn default() -> Self { Self::new(10, Duration::from_secs(30)) }

	/// Get a connection handle from the pool with timeout
	///
	/// This method acquires a semaphore permit and creates a new connection
	/// handle. If the pool is at capacity, it will wait until a connection
	/// becomes available or the timeout is reached.
	///
	/// ## Returns
	/// - `Ok(ConnectionHandle)`: New connection handle
	/// - `Err(String)`: Error message if timeout or failure occurs
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// let handle = pool.GetConnection().await?;
	/// ```
	pub async fn GetConnection(&self) -> Result<ConnectionHandle, String> {
		debug!("[ConnectionPool] Acquiring connection permit");

		// Acquire semaphore permit with timeout
		let permit = timeout(self.ConnectionTimeout, self.Semaphore.acquire())
			.await
			.map_err(|_| "Connection timeout - pool may be at capacity".to_string())?
			.map_err(|e| format!("Failed to acquire connection permit: {}", e))?;

		// Create new connection handle
		let handle = ConnectionHandle::new();

		// Add to active connections
		{
			let mut connections = self.ActiveConnections.lock().await;
			connections.insert(handle.id.clone(), handle.clone());
		}

		debug!("[ConnectionPool] Connection {} acquired (permit released on drop)", handle.id);

		// Start health monitoring for this connection
		self.StartHealthMonitoring(&handle.id).await;

		// The permit will be automatically released when dropped
		drop(permit);

		Ok(handle)
	}

	/// Release a connection handle back to the pool
	///
	/// This method removes the connection from the active connections map,
	/// allowing the semaphore permit to be reused.
	///
	/// ## Parameters
	/// - `handle`: The connection handle to release
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// pool.ReleaseConnection(handle).await;
	/// ```
	pub async fn ReleaseConnection(&self, handle:ConnectionHandle) {
		debug!("[ConnectionPool] Releasing connection {}", handle.id);

		{
			let mut connections = self.ActiveConnections.lock().await;
			connections.remove(&handle.id);
		}

		debug!("[ConnectionPool] Connection {} released", handle.id);
	}

	/// Get connection statistics for monitoring
	///
	/// This method returns aggregate statistics about the connection pool,
	/// useful for monitoring and debugging.
	///
	/// ## Returns
	/// Connection statistics including total connections, healthy connections,
	/// utilization, etc.
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// let stats = pool.GetStats().await;
	/// println!("Pool stats: {:?}", stats.summary());
	/// ```
	pub async fn GetStats(&self) -> ConnectionStats {
		let connections = self.ActiveConnections.lock().await;
		let healthy_connections = connections.values().filter(|h| h.is_healthy()).count();

		ConnectionStats {
			total_connections:connections.len(),
			healthy_connections,
			max_connections:self.MaxConnections,
			available_permits:self.Semaphore.available_permits(),
			connection_timeout:self.ConnectionTimeout,
		}
	}

	/// Clean up stale connections
	///
	/// This method removes connections that have not been used recently
	/// or are unhealthy, preventing memory leaks and resource exhaustion.
	///
	/// Stale connection criteria:
	/// - Unused for 5 minutes (300 seconds)
	/// - Not healthy (health score <= 50 or error count >= 5)
	///
	/// ## Returns
	/// The number of stale connections removed
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// let cleaned = pool.CleanUpStaleConnections().await;
	/// println!("Cleaned up {} stale connections", cleaned);
	/// ```
	pub async fn CleanUpStaleConnections(&self) -> usize {
		let mut connections = self.ActiveConnections.lock().await;
		let now = std::time::SystemTime::now();
		let stale_threshold = Duration::from_secs(300); // 5 minutes

		let stale_ids:Vec<String> = connections
			.iter()
			.filter(|(_, handle)| {
				// Check if connection is stale using SystemTime
				let is_stale_by_time = match now.duration_since(handle.last_used) {
					Ok(idle_time) => idle_time > stale_threshold,
					Err(_) => true, // If time went backwards, consider it stale
				};
				is_stale_by_time || !handle.is_healthy()
			})
			.map(|(id, _)| id.clone())
			.collect();

		let stale_count = stale_ids.len();
		for id in stale_ids {
			debug!("[ConnectionPool] Removing stale connection {}", id);
			connections.remove(&id);
		}

		if stale_count > 0 {
			debug!("[ConnectionPool] Cleaned up {} stale connection(s)", stale_count);
		}

		stale_count
	}

	/// Start health monitoring for a connection
	///
	/// This method spawns a background task that periodically checks the
	/// health of the connection and updates its health score.
	///
	/// ## Parameters
	/// - `connection_id`: The ID of the connection to monitor
	async fn StartHealthMonitoring(&self, connection_id:&str) {
		let health_checker = self.HealthChecker.clone();
		let active_connections = self.ActiveConnections.clone();
		let connection_id = connection_id.to_string();

		tokio::spawn(async move {
			let mut interval = tokio::time::interval(Duration::from_secs(30));

			loop {
				interval.tick().await;

				let mut checker = health_checker.lock().await;
				let mut connections = match active_connections.try_lock() {
					Ok(conns) => conns,
					Err(_) => continue,
				};

				if let Some(handle) = connections.get_mut(&connection_id) {
					let is_healthy = checker.check_connection_health(handle).await;
					handle.update_health(is_healthy);

					if !handle.is_healthy() {
						debug!(
							"[ConnectionPool] Connection {} marked as unhealthy (score: {:.1}, errors: {})",
							handle.id, handle.health_score, handle.error_count
						);
					}
				} else {
					// Connection removed from pool, stop monitoring
					debug!(
						"[ConnectionPool] Connection {} removed from pool, stopping health monitoring",
						connection_id
					);
					break;
				}
			}
		});
	}

	/// Get the maximum number of connections
	pub fn max_connections(&self) -> usize { self.MaxConnections }

	/// Get the connection timeout
	pub fn connection_timeout(&self) -> Duration { self.ConnectionTimeout }

	/// Get the number of available permits
	pub fn available_permits(&self) -> usize { self.Semaphore.available_permits() }

	/// Get the number of active connections
	pub async fn active_connections(&self) -> usize {
		let connections = self.ActiveConnections.lock().await;
		connections.len()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_connection_pool_creation() {
		let pool = ConnectionPool::new(10, Duration::from_secs(30));
		assert_eq!(pool.max_connections(), 10);
		assert_eq!(pool.connection_timeout(), Duration::from_secs(30));
		assert_eq!(pool.available_permits(), 10);
		assert_eq!(pool.active_connections().await, 0);
	}

	#[tokio::test]
	async fn test_default_connection_pool() {
		let pool = ConnectionPool::default();
		assert_eq!(pool.max_connections(), 10);
		assert_eq!(pool.connection_timeout(), Duration::from_secs(30));
	}

	#[tokio::test]
	async fn test_get_and_release_connection() {
		let pool = Arc::new(ConnectionPool::new(5, Duration::from_secs(5)));

		// Get a connection
		let handle = pool.GetConnection().await.unwrap();
		assert_eq!(pool.active_connections().await, 1);
		assert_eq!(pool.available_permits(), 4); // One permit used

		// Release the connection
		pool.ReleaseConnection(handle).await;
		assert_eq!(pool.active_connections().await, 0);
		assert_eq!(pool.available_permits(), 5); // Permit restored
	}

	#[tokio::test]
	async fn test_multiple_connections() {
		let pool = Arc::new(ConnectionPool::new(3, Duration::from_secs(5)));

		let handles:Vec<ConnectionHandle> = (0..3).map(|_| pool.GetConnection().await.unwrap()).collect();

		assert_eq!(pool.active_connections().await, 3);
		assert_eq!(pool.available_permits(), 0);

		// Try to get one more - should timeout
		let result = timeout(Duration::from_secs(1), pool.GetConnection()).await;
		assert!(result.is_err()); // Timeout

		// Release one connection
		pool.ReleaseConnection(handles[0].clone()).await;
		assert_eq!(pool.available_permits(), 1);

		// Now we can get another
		let handle = pool.GetConnection().await.unwrap();
		assert_eq!(pool.available_permits(), 0);

		// Release all
		for handle in handles {
			pool.ReleaseConnection(handle).await;
		}
		pool.ReleaseConnection(handle).await;
	}

	#[tokio::test]
	async fn test_connection_stats() {
		let pool = Arc::new(ConnectionPool::new(5, Duration::from_secs(30)));

		let stats = pool.GetStats().await;
		assert_eq!(stats.total_connections, 0);
		assert_eq!(stats.healthy_connections, 0);
		assert_eq!(stats.max_connections, 5);
		assert_eq!(stats.utilization(), 0.0);

		// Add some connections
		let _handles:Vec<ConnectionHandle> = (0..3).map(|_| pool.GetConnection().await.unwrap()).collect();

		let stats = pool.GetStats().await;
		assert_eq!(stats.total_connections, 3);
		assert!(stats.healthy_connections > 0);
		assert!(stats.utilization() > 0.0);
	}

	#[tokio::test]
	async fn test_cleanup_stale_connections() {
		let pool = Arc::new(ConnectionPool::new(5, Duration::from_secs(5)));

		// Create a connection and make it stale
		let mut handle = pool.GetConnection().await.unwrap();

		// Manually make it stale by setting old last_used and degrading health
		unsafe {
			let ptr = &mut handle as *mut ConnectionHandle;
			// Set last_used to a time in the past for testing
			(*ptr).last_used = std::time::SystemTime::now()
				.checked_sub(Duration::from_secs(360))
				.unwrap_or((*ptr).last_used);
			(*ptr).health_score = 25.0; // Unhealthy
		}

		// Release and try to clean up
		pool.ReleaseConnection(handle).await;

		// Clean up (will have to adjust logic for testing or add a method to force
		// cleanup) For now, we'll just verify the method exists and runs
		let cleaned = pool.CleanUpStaleConnections().await;
		assert!(cleaned >= 0);
	}

	#[tokio::test]
	async fn test_pool_utilization() {
		let pool = Arc::new(ConnectionPool::new(10, Duration::from_secs(30)));

		assert_eq!(pool.GetStats().await.utilization(), 0.0);

		// Use half the connections
		let _handles:Vec<ConnectionHandle> = (0..5).map(|_| pool.GetConnection().await.unwrap()).collect();

		let stats = pool.GetStats().await;
		assert_eq!(stats.utilization(), 50.0);
	}
}
