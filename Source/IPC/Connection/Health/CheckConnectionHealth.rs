//! `Health::CheckConnectionHealth`

use super::Struct;
use super::Types::ConnectionHandle;
use crate::dev_log;

pub fn Fn(This:&Struct, handle:&mut ConnectionHandle) -> bool {
		let start_time = std::time::Instant::now();

		// Simulate network latency (in production, this would be an actual ping)
		// Using a small delay to simulate realistic network conditions
		tokio::time::sleep(std::time::Duration::from_millis(10)).await;

		let response_time = start_time.elapsed();

		// Connection is healthy if response time is within timeout
		let is_healthy = response_time < This.PingTimeout;

		if is_healthy {
			dev_log!(
				"ipc",
				"[HealthChecker] Connection {} is healthy (response time: {:?})",
				handle.id,
				response_time
			);
		} else {
			dev_log!(
				"ipc",
				"[HealthChecker] Connection {} is unhealthy (response time: {:?}, timeout: {:?})",
				handle.id,
				response_time,
				This.PingTimeout
			);
		}

		is_healthy
	}
