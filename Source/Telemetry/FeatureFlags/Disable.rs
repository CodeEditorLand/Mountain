
//! Toggle a feature flag off through the global registry.

use crate::Telemetry::FeatureFlags::{FeatureFlagError, GlobalRegistry};

pub fn Fn(FlagName:&str, Reason:&str) -> Result<(), FeatureFlagError::Enum> {
	GlobalRegistry::REGISTRY.Disable(FlagName, Reason)
}
