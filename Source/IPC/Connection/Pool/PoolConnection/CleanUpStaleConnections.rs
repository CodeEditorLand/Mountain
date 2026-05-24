//! `PoolConnection::CleanUpStaleConnections`

use super::Struct;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
	sync::RwLock,
	time::{Instant, timeout},
};

pub fn Fn(This:&Struct) -> usize {

		let Now = Instant::now();

		let TimeoutDuration = Duration::from_secs(CLEANUP_INTERVAL_SECONDS);

		let Cleaned = {

			let mut IdleConnection = This.IdleConnection.write().await;

			let InitialCount = IdleConnection.len();

			IdleConnection.retain(|_, Handle| {
				let IdleTime = Now.duration_since(Handle.LastActivity);
				IdleTime < TimeoutDuration
			});

			InitialCount - IdleConnection.len()
		};

		Cleaned
	}
