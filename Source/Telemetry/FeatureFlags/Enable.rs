#![allow(non_snake_case)]

//! Toggle a feature flag on through the global registry.

use crate::Telemetry::FeatureFlags::{FeatureFlagError, GlobalRegistry};

pub fn Fn(FlagName:&str, Reason:&str) -> Result<(), FeatureFlagError::Enum> {
	GlobalRegistry::REGISTRY.Enable(FlagName, Reason)
}
