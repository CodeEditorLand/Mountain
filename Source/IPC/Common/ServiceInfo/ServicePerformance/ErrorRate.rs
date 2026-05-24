//! `ServicePerformance::ErrorRate`

use super::Struct;
use std::time::Instant;
use serde::Serialize;

pub fn Fn(This:&Struct) -> f64 {
		if This.RequestCount == 0 {
			return 0.0;
		}

		This.ErrorCount as f64 / This.RequestCount as f64
	}
