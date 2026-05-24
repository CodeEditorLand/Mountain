//! `ServicePerformance::RecordError`

use super::Struct;
use std::time::Instant;
use serde::Serialize;

pub fn Fn(This:&mut Struct) {
		This.ErrorCount += 1;

		This.LastUpdated = Instant::now();
	}
