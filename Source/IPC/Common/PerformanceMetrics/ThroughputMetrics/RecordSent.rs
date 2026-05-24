//! `ThroughputMetrics::RecordSent`

use super::Struct;
use std::time::Instant;

pub fn Fn(This:&mut Struct, Bytes:u64) {
		This.MessagesSent += 1;

		This.BytesSent += Bytes;
	}
