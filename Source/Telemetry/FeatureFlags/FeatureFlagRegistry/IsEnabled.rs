//! `FeatureFlagRegistry::IsEnabled`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use parking_lot::RwLock;
use crate::Telemetry::FeatureFlags::{FeatureFlag, FeatureFlagError, FlagCategory};

pub fn Fn(This:&Struct, FlagName:&str) -> bool {

		This.Flags.read().Get(FlagName).map(|F| F.Enabled).unwrap_or(false)
	}
