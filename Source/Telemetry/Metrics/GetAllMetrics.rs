#![allow(non_snake_case)]

//! Snapshot every metric currently held in the global registry.

use crate::Telemetry::Metrics::{GlobalRegistry, Metric};

pub fn Fn() -> Vec<Metric::Struct> { GlobalRegistry::REGISTRY.GetAllMetrics() }
