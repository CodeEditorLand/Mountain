//! `Pool::CleanupStaleConnections`

use std::{
	collections::HashMap,
	sync::Arc,
	time::{Duration, Instant},
};

use tokio::{
	sync::{Mutex as AsyncMutex, Notify, RwLock, Semaphore},
	time::{interval, timeout},
};

use super::Struct;
use crate::{
	IPC::Enhanced::Struct::{
		ConnectionHandle::Struct as ConnectionHandle,
		PoolConfig::Struct as PoolConfig,
		PoolStats::Struct as PoolStats,
		Struct::Struct as HealthChecker,
	},
	dev_log,
};

pub fn Fn(This:&Struct) -> usize {
	let mut connections = This.connections.lock().await;

	let stale_ids:Vec<String> = connections
		.iter()
		.filter(|(_, handle)| {
			handle.Age().as_millis() > This.config.max_lifetime_ms as u128
				|| handle.IdleTime().as_millis() > This.config.idle_timeout_ms as u128
				|| !handle.IsHealthy()
		})
		.map(|(id, _)| id.clone())
		.collect();

	for id in &stale_ids {
		connections.remove(id);
	}

	{
		let mut stats = This.stats.write().await;

		stats.total_connections = connections.len();

		stats.healthy_connections = connections.values().filter(|h| h.IsHealthy()).count();
	}

	stale_ids.len()
}
