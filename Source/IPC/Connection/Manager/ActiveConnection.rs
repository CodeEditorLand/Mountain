//! `Manager::ActiveConnection`

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
		let connections = This.ActiveConnection.lock().await;

		connections.len()
	}
