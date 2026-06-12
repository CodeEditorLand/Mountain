//! # Runtime Feature Flags
//!
//! Process-wide on/off switches that gate experimental, legacy,
//! performance-sensitive, or developer-only behaviour without recompiling.
//!
//! Layout (one export per file, file name = identity):
//! - `FeatureFlag::Struct` - a flag entry (name, state, category, reason).
//! - `FlagCategory::Enum` - Experimental / Legacy / Performance / UserFacing /
//!   Internal.
//! - `FeatureFlagError::Enum` - registry operation errors.
//! - `FeatureFlagRegistry::Struct` - thread-safe registry, seeded with the
//!   default Mountain flags.
//! - `GlobalRegistry::REGISTRY` - process-wide singleton (module-private).
//! - `IsEnabled::Fn`, `Enable::Fn`, `Disable::Fn`, `GetAllFlags::Fn`,
//!   `Initialize::Fn` - convenience accessors for the global registry.
//!
//! ## Status
//!
//! Wired up but unused as of 2026-05-02. Hydrate from `MountainEnvironment`
//! and gate flag-driven code paths.

/// Disable module.
pub mod Disable;

/// Enable module.
pub mod Enable;

/// Featureflag module.
pub mod FeatureFlag;

/// Featureflagerror module.
pub mod FeatureFlagError;

/// Featureflagregistry module.
pub mod FeatureFlagRegistry;

/// Flagcategory module.
pub mod FlagCategory;

/// Getallflags module.
pub mod GetAllFlags;

/// Initialize module.
pub mod Initialize;

/// Isenabled module.
pub mod IsEnabled;

pub(crate) mod GlobalRegistry;

#[cfg(test)]
mod tests {

	use super::{Disable, Enable, FeatureFlag, FeatureFlagRegistry, FlagCategory, IsEnabled};

	#[test]
	fn registry_creation() {
		let Registry = FeatureFlagRegistry::Struct::new();

		assert!(Registry.IsEnabled("ipc-compression"));
	}

	#[test]
	fn enable_disable() {
		let Registry = FeatureFlagRegistry::Struct::new();

		Registry.Enable("ipc-compression", "Test").unwrap();

		assert!(Registry.IsEnabled("ipc-compression"));

		Registry.Disable("ipc-compression", "Test").unwrap();

		assert!(!Registry.IsEnabled("ipc-compression"));
	}

	#[test]
	fn not_found_error() {
		let Registry = FeatureFlagRegistry::Struct::new();

		assert!(Registry.Enable("nonexistent", "Test").is_err());
	}

	#[test]
	fn add_flag() {
		let Registry = FeatureFlagRegistry::Struct::new();

		let Flag = FeatureFlag::Struct {
			Name:"test-flag".to_string(),

			Enabled:false,

			Description:"Test flag".to_string(),

			Category:FlagCategory::Enum::Experimental,

			Reason:"Testing".to_string(),
		};

		Registry.AddFlag(Flag);

		assert!(!Registry.IsEnabled("test-flag"));

		Registry.Enable("test-flag", "Test").unwrap();

		assert!(Registry.IsEnabled("test-flag"));
	}

	#[test]
	fn global_helpers() {
		let _ = IsEnabled::Fn("ipc-compression");

		let _ = Enable::Fn("ipc-compression", "test");

		let _ = Disable::Fn("ipc-compression", "test");
	}
}
