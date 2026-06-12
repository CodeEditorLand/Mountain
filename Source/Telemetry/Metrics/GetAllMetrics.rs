//! Snapshot every metric currently held in the global registry.

use crate::Telemetry::Metrics::{GlobalRegistry, Metric};

/// Public entry point for this module.
pub fn Fn() -> Vec<Metric::Struct> { GlobalRegistry::REGISTRY.GetAllMetrics() }
