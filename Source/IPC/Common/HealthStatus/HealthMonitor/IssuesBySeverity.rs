//! `HealthMonitor::IssuesBySeverity`

use super::Struct;
use std::time::Instant;
use serde::Serialize;
use crate::IPC::Common::HealthStatus::{HealthIssue, SeverityLevel};

pub fn Fn(This:&Struct, Severity:SeverityLevel::Enum) -> Vec<&HealthIssue::Enum> {
		This.Issues.iter().filter(|(_, S)| *S == Severity).map(|(I, _)| I).collect()
	}
