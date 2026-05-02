#![allow(non_snake_case)]

//! Scoped timer. Records elapsed time as a histogram against the global
//! registry on `StopAndRecord`. Labels can be attached fluently.

use std::{
	collections::HashMap,
	time::{Duration, Instant},
};

use crate::Telemetry::Metrics::GlobalRegistry;

pub struct Struct {
	Name:String,
	Labels:HashMap<String, String>,
	Start:Instant,
}

impl Struct {
	pub fn Start(Name:&str) -> Self { Self { Name:Name.to_string(), Labels:HashMap::new(), Start:Instant::now() } }

	pub fn WithLabel(mut self, Key:&str, Value:&str) -> Self {
		self.Labels.insert(Key.to_string(), Value.to_string());
		self
	}

	pub fn StopAndRecord(self) -> Duration {
		let Elapsed = self.Start.elapsed();
		GlobalRegistry::REGISTRY.RecordHistogram(&self.Name, Elapsed, self.Labels);
		Elapsed
	}
}
