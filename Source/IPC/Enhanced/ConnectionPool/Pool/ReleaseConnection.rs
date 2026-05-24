//! `Pool::ReleaseConnection`

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

pub fn Fn(This:&Struct, mut handle:ConnectionHandle) {
	let connection_id = handle.id.clone();

	handle.last_used = Instant::now();

	{
		let mut connections = This.connections.lock().await;

		connections.insert(handle.id.clone(), handle.clone());
	}

	{
		let mut stats = This.stats.write().await;

		stats.active_connections = stats.active_connections.saturating_sub(1);

		stats.idle_connections += 1;
	}

	drop(handle);

	dev_log!("ipc", "[ConnectionPool] Connection released: {}", connection_id);
}
