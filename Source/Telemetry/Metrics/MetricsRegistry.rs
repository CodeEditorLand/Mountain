//! Bounded ring buffer of recent metric observations. Oldest entries are
//! evicted FIFO once `MaxEntries` is reached. The registry is `Sync`
//! through a `parking_lot::RwLock` so emit/read can race safely.

use std::{collections::HashMap, sync::Arc, time::Duration};

use parking_lot::RwLock;

use crate::Telemetry::Metrics::{Metric, MetricValue};

#[derive(Debug)]
pub struct Struct {
	Metrics:Arc<RwLock<Vec<Metric::Struct>>>,

	MaxEntries:usize,
}

impl Struct {
	pub fn new(MaxEntries:usize) -> Self {
		Self { Metrics:Arc::new(RwLock::new(Vec::with_capacity(MaxEntries))), MaxEntries }
	}

	pub fn RecordCounter(&self, Name:&str, Value:f64, Labels:HashMap<String, String>) {
		self.Push(Metric::Struct {
			Name:Name.to_string(),
			Value:MetricValue::Enum::Counter(Value),
			Timestamp:std::time::SystemTime::now(),
			Labels,
		});
	}

	pub fn RecordGauge(&self, Name:&str, Value:f64, Labels:HashMap<String, String>) {
		self.Push(Metric::Struct {
			Name:Name.to_string(),
			Value:MetricValue::Enum::Gauge(Value),
			Timestamp:std::time::SystemTime::now(),
			Labels,
		});
	}

	pub fn RecordHistogram(&self, Name:&str, Value:Duration, Labels:HashMap<String, String>) {
		self.Push(Metric::Struct {
			Name:Name.to_string(),
			Value:MetricValue::Enum::Histogram(Value),
			Timestamp:std::time::SystemTime::now(),
			Labels,
		});
	}

	fn Push(&self, Item:Metric::Struct) {
		let mut Metrics = self.Metrics.write();

		if Metrics.len() >= self.MaxEntries {
			Metrics.remove(0);
		}

		Metrics.push(Item);
	}

	pub fn GetAllMetrics(&self) -> Vec<Metric::Struct> { self.Metrics.read().clone() }

	pub fn GetMetricsByName(&self, Name:&str) -> Vec<Metric::Struct> {
		self.Metrics.read().iter().filter(|M| M.Name == Name).cloned().collect()
	}
}
