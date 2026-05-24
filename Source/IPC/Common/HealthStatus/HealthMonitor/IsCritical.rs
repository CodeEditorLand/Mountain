//! `HealthMonitor::IsCritical`

use super::Struct;
use std::time::Instant;
use serde::Serialize;
use crate::IPC::Common::HealthStatus::{HealthIssue, SeverityLevel};

pub fn Fn(This:&Struct) -> bool { This.HealthScore < 50 }
