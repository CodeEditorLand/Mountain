//! `PoolConnection::GetStats`

use super::Struct;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
	sync::RwLock,
	time::{Instant, timeout},
};

pub fn Fn(This:&Struct) -> PoolStatistics {

		let ActiveConnection = This.ActiveConnection.read().await;

		let IdleConnection = This.IdleConnection.read().await;

		let TotalAcquired = *This.TotalAcquired.read().await;

		let TotalReleased = *This.TotalReleased.read().await;

		let ActiveCount = ActiveConnection.len();

		let IdleCount = IdleConnection.len();

		// Calculate average reuse count
		let TotalReuseCount:usize = IdleConnection
			.values()
			.chain(ActiveConnection.values())
			.map(|h| h.ReuseCount)
			.sum();

		let AverageReuseCount = if ActiveCount + IdleCount > 0 {

			TotalReuseCount as f64 / (ActiveCount + IdleCount) as f64
		} else {

			0.0
		};

		PoolStatistics {

			total_connections:ActiveCount + IdleCount,

			active_connections:ActiveCount,

			idle_connections:IdleCount,

			total_acquired:TotalAcquired,

			total_released:TotalReleased,

			average_reuse_count:AverageReuseCount,
		}
	}
