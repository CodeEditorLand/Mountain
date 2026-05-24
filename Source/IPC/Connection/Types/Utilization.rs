//! `Types::Utilization`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> f64 {
		if This.MaxConnections == 0 {
			return 0.0;
		}

		let used = This.MaxConnections - This.AvailablePermits;

		(used as f64 / This.MaxConnections as f64) * 100.0
	}
