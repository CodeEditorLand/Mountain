//! `FeatureFlagRegistry::New`

use super::Struct;
use std::{collections::HashMap, sync::Arc};
use parking_lot::RwLock;
use crate::Telemetry::FeatureFlags::{FeatureFlag, FeatureFlagError, FlagCategory};

pub fn Fn() -> Struct {

		let mut Flags = HashMap::new();

		Flags.insert(
			"ipc-compression".to_string(),

			FeatureFlag::Struct {
				Name:"ipc-compression".to_string(),
				Enabled:true,
				Description:"Enable IPC message compression".to_string(),
				Category:FlagCategory::Enum::Performance,
				Reason:"Default: improves IPC performance".to_string(),
			},
		);

		Flags.insert(
			"experimental-webgl".to_string(),

			FeatureFlag::Struct {
				Name:"experimental-webgl".to_string(),
				Enabled:false,
				Description:"Experimental WebGL rendering".to_string(),
				Category:FlagCategory::Enum::Experimental,
				Reason:"Not ready for production".to_string(),
			},
		);

		Flags.insert(
			"extension-hot-reload".to_string(),

			FeatureFlag::Struct {
				Name:"extension-hot-reload".to_string(),
				Enabled:false,
				Description:"Enable extension hot-reload".to_string(),
				Category:FlagCategory::Enum::Performance,
				Reason:"Performance impact".to_string(),
			},
		);

		Flags.insert(
			"debug-diagnostics".to_string(),

			FeatureFlag::Struct {
				Name:"debug-diagnostics".to_string(),
				Enabled:false,
				Description:"Enable detailed debug diagnostics".to_string(),
				Category:FlagCategory::Enum::Internal,
				Reason:"Development only".to_string(),
			},
		);

		Self { Flags:Arc::new(RwLock::new(Flags)) }
	}
