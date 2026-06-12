//! Snapshot every flag currently held by the global registry.

use crate::Telemetry::FeatureFlags::{FeatureFlag, GlobalRegistry};

/// fn.
pub fn Fn() -> Vec<FeatureFlag::Struct> { GlobalRegistry::REGISTRY.GetAllFlags() }
