//! `HealthMonitor::RemoveIssue`

use super::Struct;
use std::time::Instant;
use serde::Serialize;
use crate::IPC::Common::HealthStatus::{HealthIssue, SeverityLevel};

pub fn Fn(This:&mut Struct, Issue:&HealthIssue::Enum) {
		This.Issues.retain(|(I, _)| I != Issue);

		This.Recalculate();
	}
