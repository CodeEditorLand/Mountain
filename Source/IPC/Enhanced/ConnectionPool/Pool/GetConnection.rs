//! `Pool::GetConnection`

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

pub fn Fn(This:&Struct) -> Result<ConnectionHandle, String> {
	let start_time = Instant::now();

	let _permit = timeout(
		Duration::from_millis(This.config.connection_timeout_ms),
		This.semaphore.acquire(),
	)
	.await
	.map_err(|_| "Connection timeout".to_string())?
	.map_err(|E| format!("Failed to acquire connection: {}", e))?;

	let wait_time = start_time.elapsed().as_millis() as f64;

	{
		let mut stats = This.stats.write().await;

		stats.average_wait_time_ms = (stats.average_wait_time_ms * stats.total_operations as f64 + wait_time)
			/ (stats.total_operations as f64 + 1.0);
	}

	let connection = This.find_or_create_connection().await?;

	{
		let mut stats = This.stats.write().await;

		stats.active_connections += 1;

		stats.total_operations += 1;
	}

	dev_log!("ipc", "[ConnectionPool] Connection acquired: {}", connection.id);

	Ok(connection)
}
