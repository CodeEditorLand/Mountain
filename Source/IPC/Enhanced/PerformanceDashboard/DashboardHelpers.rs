//! Pure utility functions shared across `Dashboard` methods.
//!
//! All functions are deterministic and side-effect-free, making them
//! straightforward to unit-test and reuse outside the dashboard impl.

use super::MetricType::Enum as MetricType;

/// Current process memory usage in MB.
///
/// Reads the resident set size from `ps`. A missing binary or an
/// unparsable answer is a normal non-happy path: warn and report `0.0`
/// rather than failing the dashboard.
pub fn get_memory_usage() -> Result<f64, String> {
	let Pid = std::process::id().to_string();

	let Output = std::process::Command::new("ps")
		.args(["-o", "rss=", "-p", &Pid])
		.output();

	let Text = match Output {
		Ok(Output) if Output.status.success() => {
			String::from_utf8_lossy(&Output.stdout).trim().to_string()
		},

		Ok(Output) => {
			dev_log!(
				"performance",
				"warn: [DashboardHelpers] ps failed ({}); reporting 0 MB",
				String::from_utf8_lossy(&Output.stderr).trim()
			);

			return Ok(0.0);
		},

		Err(Error) => {
			dev_log!(
				"performance",
				"warn: [DashboardHelpers] ps is unavailable ({}); reporting 0 MB",
				Error
			);

			return Ok(0.0);
		},
	};

	match Text.parse::<f64>() {
		Ok(Kilobytes) => Ok(Kilobytes / 1024.0),

		Err(Error) => {
			dev_log!(
				"performance",
				"warn: [DashboardHelpers] unexpected ps rss output '{}' ({}); reporting 0 MB",
				Text,
				Error
			);

			Ok(0.0)
		},
	}
}

/// Current process CPU usage percentage.
///
/// Reads `%CPU` from `ps` and degrades to `0.0` with a warning when the
/// binary is missing or its output cannot be parsed.
pub fn get_cpu_usage() -> Result<f64, String> {
	let Pid = std::process::id().to_string();

	let Output = std::process::Command::new("ps")
		.args(["-o", "%cpu=", "-p", &Pid])
		.output();

	let Text = match Output {
		Ok(Output) if Output.status.success() => {
			String::from_utf8_lossy(&Output.stdout).trim().to_string()
		},

		Ok(Output) => {
			dev_log!(
				"performance",
				"warn: [DashboardHelpers] ps failed ({}); reporting 0% CPU",
				String::from_utf8_lossy(&Output.stderr).trim()
			);

			return Ok(0.0);
		},

		Err(Error) => {
			dev_log!(
				"performance",
				"warn: [DashboardHelpers] ps is unavailable ({}); reporting 0% CPU",
				Error
			);

			return Ok(0.0);
		},
	};

	match Text.parse::<f64>() {
		Ok(Percent) => Ok(Percent),

		Err(Error) => {
			dev_log!(
				"performance",
				"warn: [DashboardHelpers] unexpected ps cpu output '{}' ({}); reporting 0% CPU",
				Text,
				Error
			);

			Ok(0.0)
		},
	}
}

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
