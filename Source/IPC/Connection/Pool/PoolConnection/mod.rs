pub mod New;
pub mod GetConnection;
pub mod ReleaseConnection;
pub mod GetStats;
pub mod CleanUpStaleConnections;

use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
	sync::RwLock,
	time::{Instant, timeout},
};

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

/// Health status of a connection
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionHealth {

	Healthy,

	Degraded,

	Unhealthy,

/// Statistics about the connection pool
#[derive(Debug, Clone)]
pub struct PoolStatistics {

	pub total_connections:usize,

	pub active_connections:usize,

	pub idle_connections:usize,

	pub total_acquired:u64,

	pub total_released:u64,

	pub average_reuse_count:f64,

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
}
}
}

#[derive(Debug, Clone)]
pub struct Struct;
