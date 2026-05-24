//! `ServiceInfo::UpdateState`

use super::Struct;
use std::time::{Duration, Instant};
use serde::Serialize;
use crate::IPC::Common::ServiceInfo::{ServiceEndpoint, ServicePerformance, ServiceState};

pub fn Fn(This:&mut Struct, NewState:ServiceState::Enum) {
		This.State = NewState;

		This.StateSince = Instant::now();
	}
