#![allow(non_snake_case)]

//! Tagged health issue. Each variant carries a free-form description
//! string; `Severity` and `Description` accessors normalise the
//! tag→severity mapping in one place so the recalculation logic in
//! `HealthMonitor::Struct` stays a pure aggregation.

use serde::{Deserialize, Serialize};

use crate::IPC::Common::HealthStatus::SeverityLevel;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Enum {
	HighLatency(String),
	MemoryPressure(String),
	ConnectionLoss(String),
	QueueOverflow(String),
	SecurityViolation(String),
	PerformanceDegradation(String),
	Custom(String),
}

impl Enum {
	pub fn Severity(&self) -> SeverityLevel::Enum {
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

	pub fn Description(&self) -> &str {
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
}
