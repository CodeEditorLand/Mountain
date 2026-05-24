//! `PerformanceMetrics::New`

use super::Struct;
use std::time::{Duration, Instant};
use serde::Serialize;

pub fn Fn() -> Struct {
		Self {
			MessagesPerSecond:0.0,

			AverageLatencyMs:0.0,

			PeakLatencyMs:0.0,

			CompressionRatio:1.0,

			PoolUtilization:0.0,

			MemoryUsageBytes:0,

			CpuUsagePercent:0.0,

			TotalMessages:0,

			FailedMessages:0,

			LastUpdated:Instant::now(),
		}
	}
