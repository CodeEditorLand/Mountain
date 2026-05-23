//! Tunable knobs for the performance dashboard - update
//! cadence, retention window, alert threshold, sampling rate,
//! and the trace ring-buffer cap.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub update_interval_ms:u64,

	pub metrics_retention_hours:u64,

	pub alert_threshold_ms:u64,

	pub trace_sampling_rate:f64,

	pub max_traces_stored:usize,
}

impl Default for Struct {
	fn default() -> Self {
		Self {
			update_interval_ms:5000,

			metrics_retention_hours:24,

			alert_threshold_ms:1000,

			trace_sampling_rate:0.1,

			max_traces_stored:1000,
		}
	}
}
