#![allow(non_snake_case)]

//! Aggregate IPC perf snapshot: throughput, latency (avg + peak),
//! compression ratio, pool utilisation, memory + CPU usage, and
//! success/failure counters. `RecordMessage` updates the running mean
//! latency without bias under high message volume.

use std::time::{Duration, Instant};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Struct {

	pub MessagesPerSecond:f64,

	pub AverageLatencyMs:f64,

	pub PeakLatencyMs:f64,

	pub CompressionRatio:f64,

	pub PoolUtilization:f64,

	pub MemoryUsageBytes:u64,

	pub CpuUsagePercent:f64,

	pub TotalMessages:u64,

	pub FailedMessages:u64,

	#[serde(skip)]
	pub LastUpdated:Instant,
}

impl Struct {

	pub fn new() -> Self {

		Self {

			MessagesPerSecond:0.0,

			AverageLatencyMs:0.0,

			PeakLatencyMs:0.0,

			CompressionRatio:1.0,

			PoolUtilization:0.0,

			MemoryUsageBytes:0,

			CpuUsagePercent:0.0,

			TotalMessages:0,

			FailedMessages:0,

			LastUpdated:Instant::now(),
		}
	}

	pub fn RecordMessage(&mut self, Latency:Duration) {

		let LatencyMs = Latency.as_millis() as f64;

		if self.TotalMessages > 0 {

			self.AverageLatencyMs =
				(self.AverageLatencyMs * self.TotalMessages as f64 + LatencyMs) / (self.TotalMessages + 1) as f64;
		} else {

			self.AverageLatencyMs = LatencyMs;
		}

		if LatencyMs > self.PeakLatencyMs {

			self.PeakLatencyMs = LatencyMs;
		}

		self.TotalMessages += 1;

		self.LastUpdated = Instant::now();
	}

	pub fn RecordFailure(&mut self) {

		self.FailedMessages += 1;

		self.LastUpdated = Instant::now();
	}

	pub fn SuccessRate(&self) -> f64 {

		if self.TotalMessages == 0 {

			return 1.0;
		}

		1.0 - (self.FailedMessages as f64 / self.TotalMessages as f64)
	}

	pub fn IsLatencyAcceptable(&self, ThresholdMs:f64) -> bool {

		self.AverageLatencyMs <= ThresholdMs && self.PeakLatencyMs <= ThresholdMs * 2.0
	}

	pub fn SuccessRatePercent(&self) -> f64 { self.SuccessRate() * 100.0 }
}

impl Default for Struct {

	fn default() -> Self { Self::new() }
}
