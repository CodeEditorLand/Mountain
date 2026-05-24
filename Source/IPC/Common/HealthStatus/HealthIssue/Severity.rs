//! `HealthIssue::Severity`

use super::Struct;
use serde::{Deserialize, Serialize};
use crate::IPC::Common::HealthStatus::SeverityLevel;

pub fn Fn(This:&Struct) -> SeverityLevel::Enum {
		match self {
			Enum::HighLatency(_) => SeverityLevel::Enum::Medium,

			Enum::MemoryPressure(_) => SeverityLevel::Enum::Medium,

			Enum::ConnectionLoss(_) => SeverityLevel::Enum::High,

			Enum::QueueOverflow(_) => SeverityLevel::Enum::High,

			Enum::SecurityViolation(_) => SeverityLevel::Enum::Critical,

			Enum::PerformanceDegradation(_) => SeverityLevel::Enum::Medium,

			Enum::Custom(_) => SeverityLevel::Enum::Low,
		}
	}
