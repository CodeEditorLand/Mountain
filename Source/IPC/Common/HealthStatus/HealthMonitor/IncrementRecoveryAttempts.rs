//! `HealthMonitor::IncrementRecoveryAttempts`

use super::Struct;
use std::time::Instant;
use serde::Serialize;
use crate::IPC::Common::HealthStatus::{HealthIssue, SeverityLevel};

pub fn Fn(This:&mut Struct) { This.RecoveryAttempts += 1; }
