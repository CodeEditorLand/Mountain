//! Module-private singleton holding the process-wide
//! `FeatureFlagRegistry::Struct`. Convenience free functions in sibling
//! files (`IsEnabled`, `Enable`, `Disable`, `GetAllFlags`) read through
//! this static.

use std::sync::Arc;

use crate::Telemetry::FeatureFlags::FeatureFlagRegistry;

lazy_static::lazy_static! {

	pub(crate) static ref REGISTRY: Arc<FeatureFlagRegistry::Struct> =
		Arc::new(FeatureFlagRegistry::Struct::new());
}
