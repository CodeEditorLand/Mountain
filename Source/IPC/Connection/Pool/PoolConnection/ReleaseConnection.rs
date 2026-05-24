//! `PoolConnection::ReleaseConnection`

use super::Struct;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
	sync::RwLock,
	time::{Instant, timeout},
};

pub fn Fn(This:&Struct, Handle:ConnectionHandle) {

		// Remove from active connection
		let ConnectionId = Handle.ConnectionId.clone();

		{

			let mut ActiveConnection = This.ActiveConnection.write().await;

			ActiveConnection.remove(&ConnectionId);
		}

		// Return to idle if healthy
		if Handle.Health == ConnectionHealth::Healthy {

			let mut IdleConnection = This.IdleConnection.write().await;

			IdleConnection.insert(ConnectionId, Handle);
		}

		{

			let mut TotalReleased = This.TotalReleased.write().await;

			*TotalReleased += 1;
		}
	}
