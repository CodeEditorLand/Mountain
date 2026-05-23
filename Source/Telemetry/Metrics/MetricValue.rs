
//! Tagged value carried by `Metric::Struct`. Counter/Gauge/Histogram cover
//! the OTEL primitive shapes; Boolean/Text are escape hatches for
//! Mountain-internal observations that don't fit the numeric model.

use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Enum {
	/// A single numerical value that can go up or down
	Counter(f64),

	/// A single numerical value (gauge)
	Gauge(f64),

	/// A duration measurement
	Histogram(Duration),

	/// A boolean value
	Boolean(bool),

	/// A string value
	Text(String),
}
