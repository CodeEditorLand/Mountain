//! `Manager::GetStats`

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

pub fn Fn(This:&Struct) -> ConnectionStats {
		let connections = This.ActiveConnection.lock().await;

		let healthy_connections = connections.values().filter(|h| h.IsHealthy()).count();

		ConnectionStats {
			total_connections:connections.len(),

			healthy_connections,

			max_connections:This.MaxConnections,

			available_permits:This.Semaphore.AvailablePermits(),

			connection_timeout:This.ConnectionTimeout,
		}
	}
