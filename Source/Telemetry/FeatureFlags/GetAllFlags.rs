//! Snapshot every flag currently held by the global registry.

use crate::Telemetry::FeatureFlags::{FeatureFlag, GlobalRegistry};

/// Public entry point for this module.
pub fn Fn() -> Vec<FeatureFlag::Struct> { GlobalRegistry::REGISTRY.GetAllFlags() }
