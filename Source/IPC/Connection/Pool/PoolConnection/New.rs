//! `PoolConnection::New`

use super::Struct;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
	sync::RwLock,
	time::{Instant, timeout},
};

pub fn Fn(MaxConnection:usize, ConnectionTimeout:Duration) -> Struct {

		Self {

			MaxConnection:MaxConnection.min(MAX_CONNECTIONS),

			ConnectionTimeout,

			ActiveConnection:Arc::new(RwLock::new(HashMap::new())),

			IdleConnection:Arc::new(RwLock::new(HashMap::new())),

			TotalAcquired:Arc::new(RwLock::new(0)),

			TotalReleased:Arc::new(RwLock::new(0)),
		}
	}
