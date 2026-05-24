//! `Pool::New`

use super::Struct;
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
		Struct::Struct as HealthChecker,
		PoolConfig::Struct as PoolConfig,
		PoolStats::Struct as PoolStats,
	},
	dev_log,
};

pub fn Fn(config:PoolConfig) -> Struct {
		let max_connections = config.MaxConnections;

		let min_connections = config.min_connections;

		let pool = Self {
			config:config.clone(),

			connections:Arc::new(AsyncMutex::new(HashMap::new())),

			semaphore:Arc::new(Semaphore::new(max_connections)),

			wait_queue:Arc::new(AsyncMutex::new(Vec::new())),

			stats:Arc::new(RwLock::new(PoolStats {
				total_connections:0,
				active_connections:0,
				idle_connections:0,
				healthy_connections:0,
				max_connections,
				min_connections,
				wait_queue_size:0,
				average_wait_time_ms:0.0,
				total_operations:0,
				successful_operations:0,
				error_rate:0.0,
			})),

			health_checker:Arc::new(AsyncMutex::new(Struct::new())),

			is_running:Arc::new(AsyncMutex::new(false)),
		};

		dev_log!("ipc", "[ConnectionPool] Created pool with max {} connections", max_connections);

		pool
	}
