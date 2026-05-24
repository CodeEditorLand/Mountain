//! `PerformanceMetrics::RecordFailure`

use super::Struct;
use std::time::{Duration, Instant};
use serde::Serialize;

pub fn Fn(This:&mut Struct) {
		This.FailedMessages += 1;

		This.LastUpdated = Instant::now();
	}
