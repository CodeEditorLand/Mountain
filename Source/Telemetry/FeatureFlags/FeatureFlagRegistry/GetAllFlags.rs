//! `FeatureFlagRegistry::GetAllFlags`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use parking_lot::RwLock;
use crate::Telemetry::FeatureFlags::{FeatureFlag, FeatureFlagError, FlagCategory};

pub fn Fn(This:&Struct) -> Vec<FeatureFlag::Struct> { This.Flags.read().values().cloned().collect() }
