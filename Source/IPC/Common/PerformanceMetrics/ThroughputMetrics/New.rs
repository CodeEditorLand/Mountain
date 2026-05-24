//! `ThroughputMetrics::New`

use super::Struct;
use std::time::Instant;

pub fn Fn() -> Struct {
		Self {
			MessagesReceived:0,

			MessagesSent:0,

			BytesReceived:0,

			BytesSent:0,

			StartTime:Instant::now(),
		}
	}
