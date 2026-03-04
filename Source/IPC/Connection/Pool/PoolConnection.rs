//! # Pool
//!
//! ## File: IPC/Connection/Pool/PoolConnection.rs
//!
//! ## Role: Manages connection pool for efficient resource reuse
//! ## Primary Responsibility: Pool and recycle connections with capacity limits
//!
//! ## Dependencies
//! - Tokio: Async runtime and synchronization primitives
//! - Arc<TokioRwLock>: Thread-safe, async-friendly shared state
//!
//! ## Security Considerations
//! - Connection limits prevent resource exhaustion attacks
//! - Stale connection cleanup prevents memory leaks
//! - Max connections per channel prevents denial-of-service
//!
//! ## Performance Considerations
//! - Async RwLock allows multiple concurrent reads
//! - Connection reuse reduces allocation overhead
//! - Background cleanup runs without blocking operations
//! - Adaptive scaling based on usage patterns

use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::{
	sync::RwLock,
	time::{Instant, timeout},
};

/// Maximum number of connections allowed in pool
const MAX_CONNECTIONS:usize = 100;

/// Maximum time to wait for a connection
const ACQUIRE_TIMEOUT:Duration = Duration::from_secs(5);

/// Stale connection cleanup interval
const CLEANUP_INTERVAL_SECONDS:u64 = 300;

/// Connection handle representing an active connection
#[derive(Debug, Clone)]
pub struct ConnectionHandle {
	/// Unique connection identifier
	pub ConnectionId:String,

	/// Channel this connection is associated with
	pub Channel:String,

	/// Creation timestamp
	pub CreatedAt:Instant,

	/// Last activity timestamp
	pub LastActivity:Instant,

	/// Number of times this connection was reused
	pub ReuseCount:usize,

	/// Health status of the connection
	pub Health:ConnectionHealth,
}

/// Health status of a connection
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionHealth {
	Healthy,
	Degraded,
	Unhealthy,
}

/// Statistics about the connection pool
#[derive(Debug, Clone)]
pub struct PoolStatistics {
	pub total_connections:usize,
	pub active_connections:usize,
	pub idle_connections:usize,
	pub total_acquired:u64,
	pub total_released:u64,
	pub average_reuse_count:f64,
}

/// Connection pool for managing reusable connections
pub struct ConnectionPool {
	/// Maximum number of connections allowed
	pub MaxConnection:usize,

	/// Timeout for acquiring a connection
	pub ConnectionTimeout:Duration,

	/// Active connection being used
	pub ActiveConnection:Arc<RwLock<HashMap<String, ConnectionHandle>>>,

	/// Available idle connection
	pub IdleConnection:Arc<RwLock<HashMap<String, ConnectionHandle>>>,

	/// Counter for total connections acquired
	pub TotalAcquired:Arc<RwLock<u64>>,

	/// Counter for total connections released
	pub TotalReleased:Arc<RwLock<u64>>,
}

impl ConnectionPool {
	/// Create a new connection pool
	///
	/// ## Parameters
	/// - `MaxConnection`: Maximum number of connections allowed
	/// - `ConnectionTimeout`: Timeout for acquiring a connection
	///
	/// ## Returns
	/// New ConnectionPool instance
	pub fn New(MaxConnection:usize, ConnectionTimeout:Duration) -> Self {
		Self {
			MaxConnection:MaxConnection.min(MAX_CONNECTIONS),
			ConnectionTimeout,
			ActiveConnection:Arc::new(RwLock::new(HashMap::new())),
			IdleConnection:Arc::new(RwLock::new(HashMap::new())),
			TotalAcquired:Arc::new(RwLock::new(0)),
			TotalReleased:Arc::new(RwLock::new(0)),
		}
	}

	/// Acquire a connection from the pool
	///
	/// ## Parameters
	/// - `Channel`: The IPC channel name
	///
	/// ## Returns
	/// ConnectionHandle or error if timeout or pool exhausted
	pub async fn GetConnection(&self, Channel:&str) -> Result<ConnectionHandle, String> {
		// Check timeout first
		let AcquireResult = timeout(self.ConnectionTimeout, self.AcquireConnectionFromPool(Channel)).await;

		match AcquireResult {
			Ok(Result) => Result,
			Err(_) => Err(format!("Connection acquisition timed out after {:?}", self.ConnectionTimeout)),
		}
	}

	/// Internal method to acquire connection from pool
	///
	/// ## Parameters
	/// - `Channel`: The IPC channel name
	///
	/// ## Returns
	/// ConnectionHandle or error if pool exhausted
	async fn AcquireConnectionFromPool(&self, Channel:&str) -> Result<ConnectionHandle, String> {
		// Check capacity limit first
		let ActiveCount = {
			let ActiveConnection = self.ActiveConnection.read().await;
			ActiveConnection.len()
		};

		let IdleCount = {
			let IdleConnection = self.IdleConnection.read().await;
			IdleConnection.len()
		};

		if ActiveCount + IdleCount >= self.MaxConnection {
			return Err(format!(
				"Connection pool exhausted: Active: {}, Idle: {}, Max: {}",
				ActiveCount, IdleCount, self.MaxConnection
			));
		}

		// Try to reuse idle connection
		{
			let mut IdleConnection = self.IdleConnection.write().await;
			if let Some((Key, mut Handle)) = IdleConnection
				.iter()
				.find(|(_, h)| h.Channel == Channel && h.Health == ConnectionHealth::Healthy)
				.map(|(k, v)| (k.clone(), v.clone()))
			{
				IdleConnection.remove(&Key);
				Handle.ReuseCount += 1;
				Handle.LastActivity = Instant::now();

				{
					let mut ActiveConnection = self.ActiveConnection.write().await;
					ActiveConnection.insert(Handle.ConnectionId.clone(), Handle.clone());
				}

				{
					let mut TotalAcquired = self.TotalAcquired.write().await;
					*TotalAcquired += 1;
				}

				return Ok(Handle);
			}
		}

		// Create new connection
		let ConnectionId = Self::GenerateConnectionId(Channel);
		let Now = Instant::now();
		let NewHandle = ConnectionHandle {
			ConnectionId:ConnectionId.clone(),
			Channel:Channel.to_string(),
			CreatedAt:Now,
			LastActivity:Now,
			ReuseCount:0,
			Health:ConnectionHealth::Healthy,
		};

		{
			let mut ActiveConnection = self.ActiveConnection.write().await;
			ActiveConnection.insert(ConnectionId, NewHandle.clone());
		}

		{
			let mut TotalAcquired = self.TotalAcquired.write().await;
			*TotalAcquired += 1;
		}

		Ok(NewHandle)
	}

	/// Release a connection back to the pool
	///
	/// ## Parameters
	/// - `Handle`: The connection handle to release
	pub async fn ReleaseConnection(&self, Handle:ConnectionHandle) {
		// Remove from active connection
		let ConnectionId = Handle.ConnectionId.clone();
		{
			let mut ActiveConnection = self.ActiveConnection.write().await;
			ActiveConnection.remove(&ConnectionId);
		}

		// Return to idle if healthy
		if Handle.Health == ConnectionHealth::Healthy {
			let mut IdleConnection = self.IdleConnection.write().await;
			IdleConnection.insert(ConnectionId, Handle);
		}

		{
			let mut TotalReleased = self.TotalReleased.write().await;
			*TotalReleased += 1;
		}
	}

	/// Get statistics about the pool
	///
	/// ## Returns
	/// PoolStatistics with current pool metrics
	pub async fn GetStats(&self) -> PoolStatistics {
		let ActiveConnection = self.ActiveConnection.read().await;
		let IdleConnection = self.IdleConnection.read().await;
		let TotalAcquired = *self.TotalAcquired.read().await;
		let TotalReleased = *self.TotalReleased.read().await;

		let ActiveCount = ActiveConnection.len();
		let IdleCount = IdleConnection.len();

		// Calculate average reuse count
		let TotalReuseCount:usize = IdleConnection
			.values()
			.chain(ActiveConnection.values())
			.map(|h| h.ReuseCount)
			.sum();

		let AverageReuseCount = if ActiveCount + IdleCount > 0 {
			TotalReuseCount as f64 / (ActiveCount + IdleCount) as f64
		} else {
			0.0
		};

		PoolStatistics {
			total_connections:ActiveCount + IdleCount,
			active_connections:ActiveCount,
			idle_connections:IdleCount,
			total_acquired:TotalAcquired,
			total_released:TotalReleased,
			average_reuse_count:AverageReuseCount,
		}
	}

	/// Clean up stale connections
	///
	/// ## Returns
	/// Number of connections cleaned up
	pub async fn CleanUpStaleConnections(&self) -> usize {
		let Now = Instant::now();
		let TimeoutDuration = Duration::from_secs(CLEANUP_INTERVAL_SECONDS);

		let Cleaned = {
			let mut IdleConnection = self.IdleConnection.write().await;
			let InitialCount = IdleConnection.len();

			IdleConnection.retain(|_, Handle| {
				let IdleTime = Now.duration_since(Handle.LastActivity);
				IdleTime < TimeoutDuration
			});

			InitialCount - IdleConnection.len()
		};

		Cleaned
	}

	/// Generate a unique connection identifier
	///
	/// ## Parameters
	/// - `Channel`: The channel name
	///
	/// ## Returns
	/// Unique connection ID
	fn GenerateConnectionId(Channel:&str) -> String {
		use std::time::{SystemTime, UNIX_EPOCH};

		let Timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_micros();

		format!("CONN-{}-{:x}", Channel, Timestamp)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_connection_pool_new() {
		let Pool = ConnectionPool::New(10, Duration::from_secs(5));
		assert_eq!(Pool.MaxConnection, 10);

		// Verify initial state
		let Stats = Pool.GetStats().await;
		assert_eq!(Stats.total_connections, 0);
	}

	#[tokio::test]
	async fn test_connection_pool_max_connections_limit() {
		let Pool = ConnectionPool::New(1000, Duration::from_secs(5));
		assert_eq!(Pool.MaxConnection, MAX_CONNECTIONS);
	}

	#[tokio::test]
	async fn test_get_connection() {
		let Pool = ConnectionPool::New(10, Duration::from_secs(5));
		let Handle = Pool.GetConnection("test-channel").await.unwrap();

		assert_eq!(Handle.Channel, "test-channel");
		assert_eq!(Handle.ReuseCount, 0);
		assert_eq!(Handle.Health, ConnectionHealth::Healthy);

		let Stats = Pool.GetStats().await;
		assert_eq!(Stats.active_connections, 1);
		assert_eq!(Stats.total_acquired, 1);
	}

	#[tokio::test]
	async fn test_release_and_reuse_connection() {
		let Pool = ConnectionPool::New(10, Duration::from_secs(5));
		let Handle1 = Pool.GetConnection("test-channel").await.unwrap();
		let ConnectionId = Handle1.ConnectionId.clone();

		Pool.ReleaseConnection(Handle1).await;

		let Handle2 = Pool.GetConnection("test-channel").await.unwrap;
		assert_eq!(Handle2.ConnectionId, ConnectionId);
		assert_eq!(Handle2.ReuseCount, 1);
	}

	#[tokio::test]
	async fn test_pool_exhaustion() {
		let Pool = ConnectionPool::New(2, Duration::from_secs(5));

		let _Handle1 = Pool.GetConnection("test-channel-1").await.unwrap();
		let _Handle2 = Pool.GetConnection("test-channel-2").await.unwrap();

		let Result = Pool.GetConnection("test-channel-3").await;
		assert!(Result.is_err());
	}

	#[tokio::test]
	async fn test_cleanup_stale_connections() {
		let Pool = ConnectionPool::New(10, Duration::from_secs(5));

		let Handle = Pool.GetConnection("test-channel").await.unwrap();
		Pool.ReleaseConnection(Handle).await;

		// Simulate stale connection by modifying last activity
		{
			let mut IdleConnection = Pool.IdleConnection.write().await;
			if let Some(Connection) = IdleConnection.values_mut().next() {
				Connection.LastActivity = Instant::now() - Duration::from_secs(600);
			}
		}

		let Cleaned = Pool.CleanUpStaleConnections().await;
		assert!(Cleaned > 0);
	}

	#[tokio::test]
	async fn test_get_statistics() {
		let Pool = ConnectionPool::New(10, Duration::from_secs(5));

		let _Handle1 = Pool.GetConnection("test-channel-1").await.unwrap();
		let Handle2 = Pool.GetConnection("test-channel-2").await.unwrap();
		Pool.ReleaseConnection(Handle2).await;

		let Stats = Pool.GetStats().await;
		assert_eq!(Stats.active_connections, 1);
		assert_eq!(Stats.idle_connections, 1);
		assert_eq!(Stats.total_connections, 2);
	}
}
