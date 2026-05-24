//! `MetricsRegistry::RecordGauge`

use super::Struct;
use std::{collections::HashMap, sync::Arc, time::Duration};
use parking_lot::RwLock;
use crate::Telemetry::Metrics::{Metric, MetricValue};

pub fn Fn(This:&Struct, Name:&str, Value:f64, Labels:HashMap<String, String>) {
		This.push(Metric::Struct {
			Name:Name.to_string(),
			Value:MetricValue::Enum::Gauge(Value),
			Timestamp:std::time::SystemTime::now(),
			Labels,
		});
	}
