pub mod Severity;
pub mod Description;

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

#[derive(Debug, Clone)]
pub struct Struct;
