pub mod New;
pub mod Default;
pub mod GetConnection;
pub mod ReleaseConnection;
pub mod GetStats;
pub mod CleanUpStaleConnections;
pub mod MaxConnections;
pub mod ConnectionTimeout;
pub mod AvailablePermits;
pub mod ActiveConnection;

use std::{collections::HashMap, sync::Arc};
use tokio::{
	sync::{Mutex as AsyncMutex, Semaphore},
	time::{Duration, timeout},
};
use super::{
	Health::Struct,
	Types::{ConnectionHandle, ConnectionStats},
};
use crate::dev_log;

/// Connection manager (alias for ConnectionPool)
/// This is the main connection management structure, providing connection
/// pooling with health monitoring and automatic cleanup.
pub type ConnectionManager = ConnectionPool;

/// Connection pool for IPC operations
/// This structure manages a pool of connections, preventing connection
/// exhaustion by reusing connections and providing health monitoring.
/// ## Pool Architecture
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
/// ## Connection Lifecycle
/// 1. **Acquisition**: Get a connection from the pool (or create new)
/// 2. **Usage**: Use the connection for operations
/// 3. **Release**: Return the connection to the pool
/// 4. **Cleanup**: Automatically remove stale/unhealthy connections
/// ## Health Monitoring
/// Each connection has:
/// - Health score (0.0 to 100.0)
/// - Error count
/// - Last used timestamp
/// - Background health checks every 30 seconds
/// ## Example Usage
/// ```rust,ignore
/// let pool = Arc::new(ConnectionPool::new(10, Duration::from_secs(30)));
/// // Get a connection
/// let Handle = pool.GetConnection().await?;
/// // Use the connection...
/// // Release the connection
/// pool.ReleaseConnection(Handle).await;
/// // Get statistics
/// let stats = pool.GetStats().await;
/// ```
pub struct Struct {
	/// Maximum number of concurrent connections allowed
	MaxConnections:usize,

	/// Timeout for acquiring a connection from the pool
	ConnectionTimeout:Duration,

	/// Semaphore to limit concurrent connections
	Semaphore:Arc<Semaphore>,

	/// Map of active connection by ID
	ActiveConnection:Arc<AsyncMutex<HashMap<String, ConnectionHandle>>>,

	/// Health checker for monitoring connection health
	HealthChecker:Arc<AsyncMutex<HealthChecker>>,
}
