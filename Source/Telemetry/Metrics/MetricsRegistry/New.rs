//! `MetricsRegistry::New`

use super::Struct;
use std::{collections::HashMap, sync::Arc, time::Duration};
use parking_lot::RwLock;
use crate::Telemetry::Metrics::{Metric, MetricValue};

pub fn Fn(MaxEntries:usize) -> Struct {
		Self { Metrics:Arc::new(RwLock::new(Vec::with_capacity(MaxEntries))), MaxEntries }
	}
