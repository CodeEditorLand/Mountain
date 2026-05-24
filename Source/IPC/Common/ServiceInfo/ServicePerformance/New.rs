//! `ServicePerformance::New`

use super::Struct;
use std::time::Instant;
use serde::Serialize;

pub fn Fn() -> Struct {
		Self {
			RequestCount:0,

			ErrorCount:0,

			AverageResponseTimeMs:0.0,

			LastUpdated:Instant::now(),
		}
	}
