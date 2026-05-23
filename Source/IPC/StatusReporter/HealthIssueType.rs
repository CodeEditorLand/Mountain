//! Discriminator for `HealthIssue::Struct` - the kind of
//! anomaly the reporter detected during a health check.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Enum {
	HighLatency,

	MemoryPressure,

	ConnectionLoss,

	QueueOverflow,

	SecurityViolation,

	PerformanceDegradation,
}
