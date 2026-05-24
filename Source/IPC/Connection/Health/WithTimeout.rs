//! `Health::WithTimeout`

use super::Struct;
use super::Types::ConnectionHandle;
use crate::dev_log;

pub fn Fn(ping_timeout:std::time::Duration) -> Struct {
		dev_log!("ipc", "[HealthChecker] Creating health checker with {:?} timeout", ping_timeout);

		Self { ping_timeout }
	}
