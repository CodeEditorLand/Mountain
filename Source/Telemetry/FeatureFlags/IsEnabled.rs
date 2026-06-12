//! Convenience accessor for the global feature-flag registry.

use crate::Telemetry::FeatureFlags::GlobalRegistry;

/// Public entry point for this module.
pub fn Fn(FlagName:&str) -> bool { GlobalRegistry::REGISTRY.IsEnabled(FlagName) }
