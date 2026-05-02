#![allow(non_snake_case)]

//! Central thread-safe registry of `FeatureFlag::Struct` entries.
//! Backed by a `parking_lot::RwLock<HashMap>`; `new` seeds defaults
//! shipped with Mountain (compression, hot-reload, debug diagnostics, …).

use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;

use crate::Telemetry::FeatureFlags::{FeatureFlag, FeatureFlagError, FlagCategory};

#[derive(Debug)]
pub struct Struct {
	Flags:Arc<RwLock<HashMap<String, FeatureFlag::Struct>>>,
}

impl Struct {
	pub fn new() -> Self {
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

	pub fn IsEnabled(&self, FlagName:&str) -> bool {
		self.Flags.read().get(FlagName).map(|F| F.Enabled).unwrap_or(false)
	}

	pub fn Enable(&self, FlagName:&str, Reason:&str) -> Result<(), FeatureFlagError::Enum> {
		let mut Flags = self.Flags.write();
		if let Some(Flag) = Flags.get_mut(FlagName) {
			Flag.Enabled = true;
			Flag.Reason = Reason.to_string();
			Ok(())
		} else {
			Err(FeatureFlagError::Enum::NotFound(FlagName.to_string()))
		}
	}

	pub fn Disable(&self, FlagName:&str, Reason:&str) -> Result<(), FeatureFlagError::Enum> {
		let mut Flags = self.Flags.write();
		if let Some(Flag) = Flags.get_mut(FlagName) {
			Flag.Enabled = false;
			Flag.Reason = Reason.to_string();
			Ok(())
		} else {
			Err(FeatureFlagError::Enum::NotFound(FlagName.to_string()))
		}
	}

	pub fn AddFlag(&self, Flag:FeatureFlag::Struct) { self.Flags.write().insert(Flag.Name.clone(), Flag); }

	pub fn GetAllFlags(&self) -> Vec<FeatureFlag::Struct> { self.Flags.read().values().cloned().collect() }

	pub fn GetFlagsByCategory(&self, Category:FlagCategory::Enum) -> Vec<FeatureFlag::Struct> {
		self.Flags.read().values().filter(|F| F.Category == Category).cloned().collect()
	}
}
