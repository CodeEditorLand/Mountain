//! Process-wide singleton with a 10 000-entry ring buffer.

use std::sync::Arc;

use crate::Telemetry::Metrics::MetricsRegistry;

lazy_static::lazy_static! {

	pub(crate) static ref REGISTRY: Arc<MetricsRegistry::Struct> =
		Arc::new(MetricsRegistry::Struct::new(10_000));
}
