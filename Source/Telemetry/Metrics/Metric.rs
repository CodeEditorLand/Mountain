//! A single observation: name, value, capture timestamp, and free-form
//! string labels (dimensions). Stored verbatim by
//! `MetricsRegistry::Struct`.

use std::collections::HashMap;

use crate::Telemetry::Metrics::MetricValue;

#[derive(Debug, Clone)]
/// DTO for the enclosing request/response.
pub struct Struct {
	pub Name:String,

	pub Value:MetricValue::Enum,

	pub Timestamp:std::time::SystemTime,

	pub Labels:HashMap<String, String>,
}
