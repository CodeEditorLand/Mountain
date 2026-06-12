//! Inner metrics payload for `MetricsResponse`.

#[derive(Debug, Clone)]
/// DTO for the enclosing request/response.
pub struct Struct {
	pub memory_usage_mb:f64,

	pub cpu_usage_percent:f64,

	pub average_response_time:f64,

	pub disk_usage_mb:f64,

	pub network_usage_mbps:f64,
}
