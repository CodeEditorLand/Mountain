//! `Pool::ConservativePool`

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

pub fn Fn() -> Struct {
	Struct::new(PoolConfig {
		max_connections:5,
		min_connections:1,
		connection_timeout_ms:60000,
		max_lifetime_ms:600000,
		idle_timeout_ms:120000,
		health_check_interval_ms:60000,
	})
}
