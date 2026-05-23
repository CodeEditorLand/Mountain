
//! Convenience accessor for the global feature-flag registry.

use crate::Telemetry::FeatureFlags::GlobalRegistry;

pub fn Fn(FlagName:&str) -> bool { GlobalRegistry::REGISTRY.IsEnabled(FlagName) }
