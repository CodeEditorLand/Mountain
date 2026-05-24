//! `Health::SetPingTimeout`

use super::Struct;
use super::Types::ConnectionHandle;
use crate::dev_log;

pub fn Fn(This:&mut Struct, timeout:std::time::Duration) {
		This.PingTimeout = timeout;

		dev_log!("ipc", "[HealthChecker] Ping timeout updated to {:?}", timeout);
	}
