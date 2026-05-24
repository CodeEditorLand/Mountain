//! `Health::New`

use super::Struct;
use super::Types::ConnectionHandle;
use crate::dev_log;

pub fn Fn() -> Struct {
		dev_log!("ipc", "[HealthChecker] Creating health checker with 5s timeout");

		Self { ping_timeout:std::time::Duration::from_secs(5) }
	}
