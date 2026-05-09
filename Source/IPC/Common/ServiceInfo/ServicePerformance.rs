#![allow(non_snake_case)]

//! Per-service request/error counters and a rolling-mean response
//! latency. `RecordRequest` updates the running average using
//! `(prev * (n-1) + new) / n` so the value stays bounded under high
//! request volume.

use std::time::Instant;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Struct {

	pub RequestCount:u64,

	pub ErrorCount:u64,

	pub AverageResponseTimeMs:f64,

	#[serde(skip)]
	pub LastUpdated:Instant,
}

impl Struct {

	pub fn new() -> Self {

		Self {

			RequestCount:0,

			ErrorCount:0,

			AverageResponseTimeMs:0.0,

			LastUpdated:Instant::now(),
		}
	}

	pub fn RecordRequest(&mut self, ResponseTimeMs:f64) {

		self.RequestCount += 1;

		if self.AverageResponseTimeMs == 0.0 {

			self.AverageResponseTimeMs = ResponseTimeMs;
		} else {

			self.AverageResponseTimeMs = (self.AverageResponseTimeMs * (self.RequestCount - 1) as f64 + ResponseTimeMs)
				/ self.RequestCount as f64;
		}

		self.LastUpdated = Instant::now();
	}

	pub fn RecordError(&mut self) {

		self.ErrorCount += 1;

		self.LastUpdated = Instant::now();
	}

	pub fn ErrorRate(&self) -> f64 {

		if self.RequestCount == 0 {

			return 0.0;
		}

		self.ErrorCount as f64 / self.RequestCount as f64
	}
}

impl Default for Struct {

	fn default() -> Self { Self::new() }
}
