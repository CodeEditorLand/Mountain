//! `FeatureFlagRegistry::AddFlag`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use parking_lot::RwLock;
use crate::Telemetry::FeatureFlags::{FeatureFlag, FeatureFlagError, FlagCategory};

pub fn Fn(This:&Struct, Flag:FeatureFlag::Struct) { This.Flags.write().insert(Flag.Name.clone(), Flag); }
