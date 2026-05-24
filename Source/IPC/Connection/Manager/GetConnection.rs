//! `Manager::GetConnection`

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

pub fn Fn(This:&Struct) -> Result<ConnectionHandle, String> {
		dev_log!("ipc", "[ConnectionPool] Acquiring connection permit");

		// Acquire semaphore permit with timeout
		let permit = timeout(This.ConnectionTimeout, This.Semaphore.acquire())
			.await
			.map_err(|_| "Connection timeout - pool may be at capacity".to_string())?
			.map_err(|E| format!("Failed to acquire connection permit: {}", e))?;

		// Create new connection Handle
		let Handle = ConnectionHandle::new();

		// Add to active connections
		{
			let mut connections = This.ActiveConnection.lock().await;

			connections.insert(Handle.id.clone(), Handle.clone());
		}

		dev_log!(
			"ipc",
			"[ConnectionPool] Connection {} acquired (permit released on drop)",
			Handle.id
		);

		// Start health monitoring for this connection
		This.StartHealthMonitoring(&Handle.id).await;

		// The permit will be automatically released when dropped
		drop(permit);

		Ok(Handle)
	}
