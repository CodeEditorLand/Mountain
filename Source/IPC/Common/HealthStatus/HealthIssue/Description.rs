//! `HealthIssue::Description`

use super::Struct;
use serde::{Deserialize, Serialize};
use crate::IPC::Common::HealthStatus::SeverityLevel;

pub fn Fn(This:&Struct) -> &str {
		match self {
			Enum::HighLatency(D)
			| Enum::MemoryPressure(D)
			| Enum::ConnectionLoss(D)
			| Enum::QueueOverflow(D)
			| Enum::SecurityViolation(D)
			| Enum::PerformanceDegradation(D)
			| Enum::Custom(D) => D,
		}
	}
