//! Discriminator for `PerformanceMetric::Struct` - tags each
//! sample with the underlying counter so the dashboard can
//! group them.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Enum {

	MessageProcessingTime,

	ConnectionLatency,

	MemoryUsage,

	CpuUsage,

	NetworkThroughput,

	ErrorRate,

	QueueSize,
}
