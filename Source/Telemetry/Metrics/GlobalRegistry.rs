//! Process-wide singleton with a 10 000-entry ring buffer.

use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::Telemetry::Metrics::MetricsRegistry;

pub(crate) static REGISTRY:Lazy<Arc<MetricsRegistry::Struct>> =
	Lazy::new(|| Arc::new(MetricsRegistry::Struct::new(10_000)));
