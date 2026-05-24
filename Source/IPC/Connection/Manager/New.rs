//! `Manager::New`

use super::Struct;
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

pub fn Fn(MaxConnections:usize, ConnectionTimeout:Duration) -> Struct {
		dev_log!(
			"ipc",
			"[ConnectionPool] Creating pool with max: {}, timeout: {:?}",
			MaxConnections,
			ConnectionTimeout
		);

		Self {
			MaxConnections,

			ConnectionTimeout,

			Semaphore:Arc::new(Semaphore::new(MaxConnections)),

			ActiveConnection:Arc::new(AsyncMutex::new(HashMap::new())),

			HealthChecker:Arc::new(AsyncMutex::new(Struct::new())),
		}
	}
