//! Convenience: record a gauge against the global registry without labels.

use std::collections::HashMap;

use crate::Telemetry::Metrics::GlobalRegistry;

/// Public entry point for this module.
pub fn Fn(Name:&str, Value:f64) { GlobalRegistry::REGISTRY.RecordGauge(Name, Value, HashMap::new()); }
