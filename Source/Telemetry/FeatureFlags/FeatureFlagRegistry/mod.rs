pub mod New;
pub mod IsEnabled;
pub mod Enable;
pub mod Disable;
pub mod AddFlag;
pub mod GetAllFlags;
pub mod GetFlagsByCategory;

use std::{collections::HashMap, sync::Arc};
use parking_lot::RwLock;
use crate::Telemetry::FeatureFlags::{FeatureFlag, FeatureFlagError, FlagCategory};

#[derive(Debug)]
pub struct Struct {

	Flags:Arc<RwLock<HashMap<String, FeatureFlag::Struct>>>,
}
