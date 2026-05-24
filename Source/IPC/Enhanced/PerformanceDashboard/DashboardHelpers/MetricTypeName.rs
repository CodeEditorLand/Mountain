//! `DashboardHelpers::MetricTypeName`

use super::MetricType::Enum as MetricType;

/// Human-readable display name for a `MetricType` variant.
pub fn Fn(metric_type:&MetricType) -> &'static str {
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
