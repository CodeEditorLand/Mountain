//! `HealthMonitor::ClearIssues`

use super::Struct;
use std::time::Instant;
use serde::Serialize;
use crate::IPC::Common::HealthStatus::{HealthIssue, SeverityLevel};

pub fn Fn(This:&mut Struct) {
		This.Issues.clear();

		This.HealthScore = 100;

		This.LastCheck = Instant::now();
	}
