//! `Pool::Stop`

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

pub fn Fn(This:&Struct) -> Result<(), String> {
		{
			let mut running = This.IsRunning.lock().await;

			if !*running {
				return Ok(());
			}

			*running = false;
		}

		{
			let mut connections = This.connections.lock().await;

			connections.clear();
		}

		{
			let mut wait_queue = This.wait_queue.lock().await;

			for notifier in wait_queue.drain(..) {
				notifier.notify_one();
			}
		}

		dev_log!("ipc", "[ConnectionPool] Stopped connection pool");

		Ok(())
	}
