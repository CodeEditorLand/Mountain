//! `PerformanceMetrics::IsLatencyAcceptable`

use super::Struct;
use std::time::{Duration, Instant};
use serde::Serialize;

pub fn Fn(This:&Struct, ThresholdMs:f64) -> bool {
		This.AverageLatencyMs <= ThresholdMs && This.PeakLatencyMs <= ThresholdMs * 2.0
	}
