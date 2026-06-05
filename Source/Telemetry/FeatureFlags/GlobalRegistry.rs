//! Module-private singleton holding the process-wide
//! `FeatureFlagRegistry::Struct`. Convenience free functions in sibling
//! files (`IsEnabled`, `Enable`, `Disable`, `GetAllFlags`) read through
//! this static.

use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::Telemetry::FeatureFlags::FeatureFlagRegistry;

pub(crate) static REGISTRY:Lazy<Arc<FeatureFlagRegistry::Struct>> =
	Lazy::new(|| Arc::new(FeatureFlagRegistry::Struct::new()));
