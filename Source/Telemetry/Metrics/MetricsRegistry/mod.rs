pub mod New;
pub mod RecordCounter;
pub mod RecordGauge;
pub mod RecordHistogram;
pub mod GetAllMetrics;
pub mod GetMetricsByName;

use std::{collections::HashMap, sync::Arc, time::Duration};
use parking_lot::RwLock;
use crate::Telemetry::Metrics::{Metric, MetricValue};

#[derive(Debug)]
pub struct Struct {
	Metrics:Arc<RwLock<Vec<Metric::Struct>>>,

	MaxEntries:usize,
}
