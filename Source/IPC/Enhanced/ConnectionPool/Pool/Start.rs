//! `Pool::Start`

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

pub fn Fn(This:&Struct) -> Result<(), String> {
	{
		let mut running = This.IsRunning.lock().await;

		if *running {
			return Ok(());
		}

		*running = true;
	}

	This.start_health_monitoring().await;

	This.start_connection_cleanup().await;

	This.initialize_min_connections().await;

	dev_log!("ipc", "[ConnectionPool] Started connection pool");

	Ok(())
}
