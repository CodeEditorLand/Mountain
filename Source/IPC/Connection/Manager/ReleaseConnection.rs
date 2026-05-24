//! `Manager::ReleaseConnection`

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

pub fn Fn(This:&Struct, Handle:ConnectionHandle) {
		dev_log!("ipc", "[ConnectionPool] Releasing connection {}", Handle.id);

		{
			let mut connections = This.ActiveConnection.lock().await;

			connections.remove(&Handle.id);
		}

		dev_log!("ipc", "[ConnectionPool] Connection {} released", Handle.id);
	}
