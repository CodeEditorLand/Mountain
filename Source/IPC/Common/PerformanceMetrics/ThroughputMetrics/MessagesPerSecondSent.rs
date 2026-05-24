//! `ThroughputMetrics::MessagesPerSecondSent`

use super::Struct;
use std::time::Instant;

pub fn Fn(This:&Struct) -> f64 {
		let Elapsed = This.StartTime.elapsed().as_secs_f64();

		if Elapsed > 0.0 { This.MessagesSent as f64 / Elapsed } else { 0.0 }
	}
