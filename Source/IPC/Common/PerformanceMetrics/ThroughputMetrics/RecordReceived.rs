//! `ThroughputMetrics::RecordReceived`

use super::Struct;
use std::time::Instant;

pub fn Fn(This:&mut Struct, Bytes:u64) {
		This.MessagesReceived += 1;

		This.BytesReceived += Bytes;
	}
