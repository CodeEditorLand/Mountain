//! # Runtime Feature Flags
//!
//! This module provides runtime feature flag management for the Mountain
//! application. Feature flags allow enabling or disabling features without
//! recompiling the application.
//!
//! ## Feature Flag Categories
//!
//! - **Experimental Features**: New features under development
//! - **Legacy Features**: Old features that can be disabled
//! - **Performance Features**: Features that impact performance
//! - **User-facing Features**: Features visible to end users
//!
//! ## Usage Example
//!
//! ```rust
//! use Mountain::Telemetry::FeatureFlags;
//!
//! // Check if a feature is enabled
//! if FeatureFlags::is_enabled("experimental-ui") {
//! 	show_experimental_ui();
//! }
//!
//! // Enable a feature
//! FeatureFlags::enable_feature("custom-editor", "User preference");
//! ```

use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;

// ============================================================================
// Feature Flag Data Structures
// ============================================================================

/// A feature flag with metadata
#[derive(Debug, Clone)]
pub struct FeatureFlag {
	pub name:String,
	pub enabled:bool,
	pub description:String,
	pub category:FlagCategory,
	pub reason:String,
}

/// Categories of feature flags
#[derive(Debug, Clone, PartialEq)]
pub enum FlagCategory {
	/// Experimental features (may change or be removed)
	Experimental,
	/// Legacy features (can be disabled)
	Legacy,
	/// Performance-sensitive features
	Performance,
	/// User-facing features
	UserFacing,
	/// Internal/developer features
	Internal,
}

// ============================================================================
// Feature Flag Registry
// ============================================================================

/// Central registry for feature flags
#[derive(Debug)]
pub struct FeatureFlagRegistry {
	flags:Arc<RwLock<HashMap<String, FeatureFlag>>>,
}

impl FeatureFlagRegistry {
	/// Create a new registry with default feature flags
	pub fn new() -> Self {
		let mut flags = HashMap::new();

		// Add default feature flags
		flags.insert(
			"ipc-compression".to_string(),
			FeatureFlag {
				name:"ipc-compression".to_string(),
				enabled:true,
				description:"Enable IPC message compression".to_string(),
				category:FlagCategory::Performance,
				reason:"Default: improves IPC performance".to_string(),
			},
		);

		flags.insert(
			"experimental-webgl".to_string(),
			FeatureFlag {
				name:"experimental-webgl".to_string(),
				enabled:false,
				description:"Experimental WebGL rendering".to_string(),
				category:FlagCategory::Experimental,
				reason:"Not ready for production".to_string(),
			},
		);

		flags.insert(
			"extension-hot-reload".to_string(),
			FeatureFlag {
				name:"extension-hot-reload".to_string(),
				enabled:false,
				description:"Enable extension hot-reload".to_string(),
				category:FlagCategory::Performance,
				reason:"Performance impact".to_string(),
			},
		);

		flags.insert(
			"debug-diagnostics".to_string(),
			FeatureFlag {
				name:"debug-diagnostics".to_string(),
				enabled:false,
				description:"Enable detailed debug diagnostics".to_string(),
				category:FlagCategory::Internal,
				reason:"Development only".to_string(),
			},
		);

		Self { flags:Arc::new(RwLock::new(flags)) }
	}

	/// Check if a feature is enabled
	pub fn is_enabled(&self, flag_name:&str) -> bool {
		self.flags.read().get(flag_name).map(|f| f.enabled).unwrap_or(false)
	}

	/// Enable a feature flag
	pub fn enable(&self, flag_name:&str, reason:&str) -> Result<(), FeatureFlagError> {
		let mut flags = self.flags.write();

		if let Some(flag) = flags.get_mut(flag_name) {
			flag.enabled = true;
			flag.reason = reason.to_string();
			Ok(())
		} else {
			Err(FeatureFlagError::NotFound(flag_name.to_string()))
		}
	}

	/// Disable a feature flag
	pub fn disable(&self, flag_name:&str, reason:&str) -> Result<(), FeatureFlagError> {
		let mut flags = self.flags.write();

		if let Some(flag) = flags.get_mut(flag_name) {
			flag.enabled = false;
			flag.reason = reason.to_string();
			Ok(())
		} else {
			Err(FeatureFlagError::NotFound(flag_name.to_string()))
		}
	}

	/// Add a new feature flag
	pub fn add_flag(&self, flag:FeatureFlag) {
		let mut flags = self.flags.write();
		flags.insert(flag.name.clone(), flag);
	}

	/// Get all feature flags
	pub fn get_all_flags(&self) -> Vec<FeatureFlag> { self.flags.read().values().cloned().collect() }

	/// Get flags by category
	pub fn get_flags_by_category(&self, category:FlagCategory) -> Vec<FeatureFlag> {
		self.flags.read().values().filter(|f| f.category == category).cloned().collect()
	}
}

/// Error types for feature flag operations
#[derive(Debug, thiserror::Error)]
pub enum FeatureFlagError {
	#[error("Feature flag not found: {0}")]
	NotFound(String),
	#[error("Feature flag already exists: {0}")]
	AlreadyExists(String),
	#[error("Feature flag error: {0}")]
	Other(String),
}

/// Global feature flag registry instance
lazy_static::lazy_static! {
	static ref GLOBAL_REGISTRY: Arc<FeatureFlagRegistry> =
		Arc::new(FeatureFlagRegistry::new());
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Check if a feature flag is enabled
pub fn is_enabled(flag_name:&str) -> bool { GLOBAL_REGISTRY.is_enabled(flag_name) }

/// Enable a feature flag
pub fn enable(flag_name:&str, reason:&str) -> Result<(), FeatureFlagError> { GLOBAL_REGISTRY.enable(flag_name, reason) }

/// Disable a feature flag
pub fn disable(flag_name:&str, reason:&str) -> Result<(), FeatureFlagError> {
	GLOBAL_REGISTRY.disable(flag_name, reason)
}

/// Get all feature flags
pub fn get_all_flags() -> Vec<FeatureFlag> { GLOBAL_REGISTRY.get_all_flags() }

// ============================================================================
// Initialization
// ============================================================================

/// Initialize feature flags from configuration
pub fn initialize_feature_flags() -> Result<(), FeatureFlagError> {
	log::debug!("Feature flags system initialized");
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_registry_creation() {
		let registry = FeatureFlagRegistry::new();
		assert!(registry.is_enabled("ipc-compression"));
	}

	#[test]
	fn test_enable_disable() {
		let registry = FeatureFlagRegistry::new();

		registry.enable("ipc-compression", "Test").unwrap();
		assert!(registry.is_enabled("ipc-compression"));

		registry.disable("ipc-compression", "Test").unwrap();
		assert!(!registry.is_enabled("ipc-compression"));
	}

	#[test]
	fn test_not_found_error() {
		let registry = FeatureFlagRegistry::new();
		let result = registry.enable("nonexistent", "Test");
		assert!(result.is_err());
	}

	#[test]
	fn test_add_flag() {
		let registry = FeatureFlagRegistry::new();

		let flag = FeatureFlag {
			name:"test-flag".to_string(),
			enabled:false,
			description:"Test flag".to_string(),
			category:FlagCategory::Experimental,
			reason:"Testing".to_string(),
		};

		registry.add_flag(flag);
		assert!(!registry.is_enabled("test-flag"));

		registry.enable("test-flag", "Test").unwrap();
		assert!(registry.is_enabled("test-flag"));
	}
}
