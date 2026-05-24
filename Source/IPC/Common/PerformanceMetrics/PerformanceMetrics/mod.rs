pub mod New;
pub mod RecordMessage;
pub mod RecordFailure;
pub mod SuccessRate;
pub mod IsLatencyAcceptable;
pub mod SuccessRatePercent;

use std::time::{Duration, Instant};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Struct {
	pub MessagesPerSecond:f64,

	pub AverageLatencyMs:f64,

	pub PeakLatencyMs:f64,

	pub CompressionRatio:f64,

	pub PoolUtilization:f64,

	pub MemoryUsageBytes:u64,

	pub CpuUsagePercent:f64,

	pub TotalMessages:u64,

	pub FailedMessages:u64,

	#[serde(skip)]
	pub LastUpdated:Instant,
}
