#![allow(non_snake_case)]

//! Air daemon resource metrics DTO returned by `GetAirMetrics`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub memory_usage_mb:f64,

	pub cpu_usage_percent:f64,

	pub average_response_time:f64,

	pub disk_usage_mb:f64,

	pub network_usage_mbps:f64,
}
