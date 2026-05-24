//! `ServiceInfo::RecordHeartbeat`

use super::Struct;
use std::time::{Duration, Instant};
use serde::Serialize;
use crate::IPC::Common::ServiceInfo::{ServiceEndpoint, ServicePerformance, ServiceState};

pub fn Fn(This:&mut Struct) {
		This.LastHeartbeat = Some(Instant::now());

		if This.State == ServiceState::Enum::Running {
			This.Uptime = This.StateSince.elapsed();
		}
	}
