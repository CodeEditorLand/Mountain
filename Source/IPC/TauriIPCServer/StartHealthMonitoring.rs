//! Spawns the 30-second-interval health monitor task for a pooled
//! connection. Body of `ConnectionPool::StartHealthMonitoring`.

use std::time::Duration;

use super::ConnectionPool;
use crate::dev_log;

pub(crate) async fn Fn(Pool:&ConnectionPool, connection_id:&str) {
	let health_checker = Pool.HealthChecker.clone();

	let active_connection = Pool.ActiveConnection.clone();

	let connection_id = connection_id.to_string();

	tokio::spawn(async move {
		let mut interval = tokio::time::interval(Duration::from_secs(30));

		loop {
			interval.tick().await;

			let checker = health_checker.lock().await;

			let mut connections = match active_connection.try_lock() {
				Ok(conns) => conns,
				Err(_) => continue,
			};

			if let Some(Handle) = connections.get_mut(&connection_id) {
				let is_healthy = checker.check_connection_health(Handle).await;

				Handle.update_health(is_healthy);

				if !Handle.is_healthy() {
					dev_log!(
						"ipc",
						"Connection {} marked as unhealthy (score: {:.1})",
						Handle.id,
						Handle.health_score
					);
				}
			} else {
				// The connection has been removed from the pool, stop monitoring
				break;
			}
		}
	});
}
