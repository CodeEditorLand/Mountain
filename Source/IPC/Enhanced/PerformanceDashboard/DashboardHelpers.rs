//! Pure utility functions shared across `Dashboard` methods.
//!
//! All functions are deterministic and side-effect-free, making them
//! straightforward to unit-test and reuse outside the dashboard impl.

use super::MetricType::Enum as MetricType;

/// Current process memory usage in MB (stub: returns 100.0 until real
/// platform metrics are wired).
pub fn get_memory_usage() -> Result<f64, String> { Ok(100.0) }

/// Current process CPU usage percentage (stub: returns 25.0 until real
/// platform metrics are wired).
pub fn get_cpu_usage() -> Result<f64, String> { Ok(25.0) }

/// Generate a new UUID v4 string for use as a trace identifier.
pub fn generate_trace_id() -> String { uuid::Uuid::new_v4().to_string() }

/// Generate a new UUID v4 string for use as a span identifier.
pub fn generate_span_id() -> String { uuid::Uuid::new_v4().to_string() }

/// Generate a new UUID v4 string for use as an alert identifier.
pub fn generate_alert_id() -> String { uuid::Uuid::new_v4().to_string() }

/// Human-readable display name for a `MetricType` variant.
pub fn metric_type_name(metric_type:&MetricType) -> &'static str {
	match metric_type {
		MetricType::MessageProcessingTime => "Message Processing Time",

		MetricType::ConnectionLatency => "Connection Latency",

		MetricType::MemoryUsage => "Memory Usage",

		MetricType::CpuUsage => "CPU Usage",

		MetricType::NetworkThroughput => "Network Throughput",

		MetricType::ErrorRate => "Error Rate",

		MetricType::QueueSize => "Queue Size",
	}
}
