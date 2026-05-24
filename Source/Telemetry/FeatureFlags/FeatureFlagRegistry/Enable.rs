//! `FeatureFlagRegistry::Enable`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use parking_lot::RwLock;
use crate::Telemetry::FeatureFlags::{FeatureFlag, FeatureFlagError, FlagCategory};

pub fn Fn(This:&Struct, FlagName:&str, Reason:&str) -> Result<(), FeatureFlagError::Enum> {

		let mut Flags = This.Flags.write();

		if let Some(Flag) = Flags.get_mut(FlagName) {

			Flag.Enabled = true;

			Flag.Reason = Reason.to_string();

			Ok(())
		} else {

			Err(FeatureFlagError::Enum::NotFound(FlagName.to_string()))
		}
	}
