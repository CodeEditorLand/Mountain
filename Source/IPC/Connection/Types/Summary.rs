//! `Types::Summary`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> String {
		format!(
			"Connections: {}/{} ({}%), Healthy: {}%, Utilization: {}%",
			This.total_connections,
			This.MaxConnections,
			This.HealthPercentage(),
			This.HealthPercentage(),
			This.Utilization()
		)
	}
