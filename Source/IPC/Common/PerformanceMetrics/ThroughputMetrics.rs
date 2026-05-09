#![allow(non_snake_case)]

//! Per-direction message + byte counters with a fixed start time so
//! `MessagesPerSecond*` / `BytesPerSecond*` are derivable as
//! divisions over the elapsed period.

use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Struct {

	pub MessagesReceived:u64,

	pub MessagesSent:u64,

	pub BytesReceived:u64,

	pub BytesSent:u64,

	pub StartTime:Instant,
}

impl Struct {

	pub fn new() -> Self {

		Self {

			MessagesReceived:0,

			MessagesSent:0,

			BytesReceived:0,

			BytesSent:0,

			StartTime:Instant::now(),
		}
	}

	pub fn RecordReceived(&mut self, Bytes:u64) {

		self.MessagesReceived += 1;

		self.BytesReceived += Bytes;
	}

	pub fn RecordSent(&mut self, Bytes:u64) {

		self.MessagesSent += 1;

		self.BytesSent += Bytes;
	}

	pub fn MessagesPerSecondReceived(&self) -> f64 {

		let Elapsed = self.StartTime.elapsed().as_secs_f64();

		if Elapsed > 0.0 { self.MessagesReceived as f64 / Elapsed } else { 0.0 }
	}

	pub fn MessagesPerSecondSent(&self) -> f64 {

		let Elapsed = self.StartTime.elapsed().as_secs_f64();

		if Elapsed > 0.0 { self.MessagesSent as f64 / Elapsed } else { 0.0 }
	}

	pub fn BytesPerSecondReceived(&self) -> f64 {

		let Elapsed = self.StartTime.elapsed().as_secs_f64();

		if Elapsed > 0.0 { self.BytesReceived as f64 / Elapsed } else { 0.0 }
	}

	pub fn BytesPerSecondSent(&self) -> f64 {

		let Elapsed = self.StartTime.elapsed().as_secs_f64();

		if Elapsed > 0.0 { self.BytesSent as f64 / Elapsed } else { 0.0 }
	}
}

impl Default for Struct {

	fn default() -> Self { Self::new() }
}
