//! `PoolConnection::GetConnection`

use super::Struct;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
	sync::RwLock,
	time::{Instant, timeout},
};

pub fn Fn(This:&Struct, Channel:&str) -> Result<ConnectionHandle, String> {

		// Check timeout first
		let AcquireResult = timeout(This.ConnectionTimeout, This.AcquireConnectionFromPool(Channel)).await;

		match AcquireResult {

			Ok(Result) => Result,

			Err(_) => Err(format!("Connection acquisition timed out after {:?}", This.ConnectionTimeout)),
		}
	}
