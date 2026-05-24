//! `PerformanceMetrics::SuccessRate`

use super::Struct;
use std::time::{Duration, Instant};
use serde::Serialize;

pub fn Fn(This:&Struct) -> f64 {
		if This.TotalMessages == 0 {
			return 1.0;
		}

		1.0 - (This.FailedMessages as f64 / This.TotalMessages as f64)
	}
