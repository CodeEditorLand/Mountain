//! `MetricsRegistry::GetMetricsByName`

use super::Struct;
use std::{collections::HashMap, sync::Arc, time::Duration};
use parking_lot::RwLock;
use crate::Telemetry::Metrics::{Metric, MetricValue};

pub fn Fn(This:&Struct, Name:&str) -> Vec<Metric::Struct> {
		This.Metrics.read().iter().filter(|M| M.Name == Name).cloned().collect()
	}
