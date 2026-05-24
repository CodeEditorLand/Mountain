//! `PerformanceMetrics::RecordMessage`

use super::Struct;
use std::time::{Duration, Instant};
use serde::Serialize;

pub fn Fn(This:&mut Struct, Latency:Duration) {
		let LatencyMs = Latency.as_millis() as f64;

		if This.TotalMessages > 0 {
			This.AverageLatencyMs =
				(This.AverageLatencyMs * This.TotalMessages as f64 + LatencyMs) / (This.TotalMessages + 1) as f64;
		} else {
			This.AverageLatencyMs = LatencyMs;
		}

		if LatencyMs > This.PeakLatencyMs {
			This.PeakLatencyMs = LatencyMs;
		}

		This.TotalMessages += 1;

		This.LastUpdated = Instant::now();
	}
