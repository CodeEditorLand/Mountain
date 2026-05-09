#![allow(non_snake_case)]

//! Threshold-violation alert raised by the dashboard - what
//! tripped, current vs threshold, severity, human-readable
//! message.

use serde::{Deserialize, Serialize};

use crate::IPC::Enhanced::PerformanceDashboard::{AlertSeverity, MetricType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub alert_id:String,

	pub metric_type:MetricType::Enum,

	pub threshold:f64,

	pub current_value:f64,

	pub timestamp:u64,

	pub channel:Option<String>,

	pub severity:AlertSeverity::Enum,

	pub message:String,
}
