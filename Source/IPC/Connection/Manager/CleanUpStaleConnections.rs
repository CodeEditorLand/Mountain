//! `Manager::CleanUpStaleConnections`

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

pub fn Fn(This:&Struct) -> usize {
		let mut connections = This.ActiveConnection.lock().await;

		let now = std::time::SystemTime::now();

		let stale_threshold = Duration::from_secs(300); // 5 minutes

		let stale_ids:Vec<String> = connections
			.iter()
			.filter(|(_, Handle)| {
				// Check if connection is stale using SystemTime
				let is_stale_by_time = match now.duration_since(Handle.last_used) {
					Ok(idle_time) => idle_time > stale_threshold,
					Err(_) => true, // If time went backwards, consider it stale
				};
				is_stale_by_time || !Handle.IsHealthy()
			})
			.map(|(id, _)| id.clone())
			.collect();

		let stale_count = stale_ids.len();

		for id in stale_ids {
			dev_log!("ipc", "[ConnectionPool] Removing stale connection {}", id);

			connections.remove(&id);
		}

		if stale_count > 0 {
			dev_log!("ipc", "[ConnectionPool] Cleaned up {} stale connection(s)", stale_count);
		}

		stale_count
	}
