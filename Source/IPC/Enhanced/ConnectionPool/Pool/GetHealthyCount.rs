//! `Pool::GetHealthyCount`

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

pub fn Fn(This:&Struct) -> usize { This.stats.read().await.healthy_connections }
