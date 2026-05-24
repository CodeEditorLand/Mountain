//! `HealthMonitor::AddIssue`

use super::Struct;
use std::time::Instant;
use serde::Serialize;
use crate::IPC::Common::HealthStatus::{HealthIssue, SeverityLevel};

pub fn Fn(This:&mut Struct, Issue:HealthIssue::Enum) {
		let Severity = Issue.Severity();

		This.Issues.push((Issue, Severity));

		This.Recalculate();
	}
