//! One sample point: typed metric, value, timestamp,
//! optional channel, free-form tag bag.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::IPC::Enhanced::PerformanceDashboard::MetricType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {

	pub metric_type:MetricType::Enum,

	pub value:f64,

	pub timestamp:u64,

	pub channel:Option<String>,

	pub tags:HashMap<String, String>,
}
