pub mod New;
pub mod Start;
pub mod Stop;
pub mod GetConnection;
pub mod ReleaseConnection;
pub mod CleanupStaleConnections;
pub mod GetStats;
pub mod GetActiveCount;
pub mod GetHealthyCount;
pub mod IsRunning;
pub mod DefaultPool;
pub mod HighPerformancePool;
pub mod ConservativePool;
pub mod CalculateOptimalPoolSize;

use std::{
	collections::HashMap,
	sync::Arc,
	time::{Duration, Instant},
};

use tokio::{
	sync::{Mutex as AsyncMutex, Notify, RwLock, Semaphore},
	time::{interval, timeout},
};

use crate::{
	IPC::Enhanced::Struct::{
		ConnectionHandle::Struct as ConnectionHandle,
		PoolConfig::Struct as PoolConfig,
		PoolStats::Struct as PoolStats,
		Struct::Struct as HealthChecker,
	},
	dev_log,
};

pub struct Struct {
	pub config:PoolConfig,

	pub connections:Arc<AsyncMutex<HashMap<String, ConnectionHandle>>>,

	pub semaphore:Arc<Semaphore>,

	pub wait_queue:Arc<AsyncMutex<Vec<Arc<Notify>>>>,

	pub stats:Arc<RwLock<PoolStats>>,

	pub health_checker:Arc<AsyncMutex<HealthChecker>>,

	pub is_running:Arc<AsyncMutex<bool>>,
}
