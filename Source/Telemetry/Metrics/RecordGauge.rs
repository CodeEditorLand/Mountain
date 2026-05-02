#![allow(non_snake_case)]

//! Convenience: record a gauge against the global registry without labels.

use std::collections::HashMap;

use crate::Telemetry::Metrics::GlobalRegistry;

pub fn Fn(Name:&str, Value:f64) { GlobalRegistry::REGISTRY.RecordGauge(Name, Value, HashMap::new()); }
