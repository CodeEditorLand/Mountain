//! # Telemetry RPC Service
//!
//! OTEL integration for telemetry collection.

use serde::{Deserialize, Serialize};

/// Telemetry service
pub struct TelemetryService;

impl TelemetryService {
	pub fn new() -> Self { Self {} }
}

impl Default for TelemetryService {
	fn default() -> Self { Self::new() }
}

/// Telemetry span
pub mod spans {
	use super::*;

	/// Trace span
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct TraceSpan {
		pub trace_id:String,
		pub span_id:String,
		pub parent_span_id:Option<String>,
		pub name:String,
		pub start_time:i64,
		pub end_time:Option<i64>,
	}
}

/// Service metrics
pub mod metrics {
	use super::*;

	/// Service metrics
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct ServiceMetrics {
		pub name:String,
		pub count:u64,
		pub sum:f64,
	}
}
