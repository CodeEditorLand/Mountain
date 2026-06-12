//! Eager initialiser for the feature-flag system. Currently a no-op log
//! line; in future phases this will hydrate flags from
//! `MountainEnvironment` configuration.

use crate::{Telemetry::FeatureFlags::FeatureFlagError, dev_log};

/// fn.
pub fn Fn() -> Result<(), FeatureFlagError::Enum> {
	dev_log!("config", "feature flags system initialized");

	Ok(())
}
