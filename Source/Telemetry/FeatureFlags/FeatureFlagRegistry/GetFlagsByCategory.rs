//! `FeatureFlagRegistry::GetFlagsByCategory`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use parking_lot::RwLock;
use crate::Telemetry::FeatureFlags::{FeatureFlag, FeatureFlagError, FlagCategory};

pub fn Fn(This:&Struct, Category:FlagCategory::Enum) -> Vec<FeatureFlag::Struct> {

		This.Flags.read().values().filter(|F| F.Category == Category).cloned().collect()
	}
