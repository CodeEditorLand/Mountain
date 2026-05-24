//! `ServiceInfo::IsHealthy`

use super::Struct;
use std::time::{Duration, Instant};
use serde::Serialize;
use crate::IPC::Common::ServiceInfo::{ServiceEndpoint, ServicePerformance, ServiceState};

pub fn Fn(This:&Struct) -> bool {
		if !This.State.IsOperational() {
			return false;
		}

		if let Some(Heartbeat) = This.LastHeartbeat
			&& Heartbeat.elapsed() > Duration::from_secs(30)
		{
			return false;
		}

		if This.Performance.ErrorRate() > 0.1 {
			return false;
		}

		true
	}
