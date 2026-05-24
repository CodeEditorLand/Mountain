//! `ServicePerformance::RecordRequest`

use super::Struct;
use std::time::Instant;
use serde::Serialize;

pub fn Fn(This:&mut Struct, ResponseTimeMs:f64) {
		This.RequestCount += 1;

		if This.AverageResponseTimeMs == 0.0 {
			This.AverageResponseTimeMs = ResponseTimeMs;
		} else {
			This.AverageResponseTimeMs = (This.AverageResponseTimeMs * (This.RequestCount - 1) as f64 + ResponseTimeMs)
				/ This.RequestCount as f64;
		}

		This.LastUpdated = Instant::now();
	}
