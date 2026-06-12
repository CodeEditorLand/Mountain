//! # ConfigurationProvider (Environment)
//!
//! Implements `ConfigurationProvider` and `ConfigurationInspector` traits,
//! managing all application settings across multiple scopes (Default, User,
//! Workspace, Folder). It handles the configuration cascade, merging settings
//! from various sources in the correct precedence order.
//!
//! ## Implementation Strategy
//!
//! The trait implementation is split across multiple helper modules for
//! maintainability:
//! - `GetValue`: `GetConfigurationValue` - retrieval from merged cache
//! - `UpdateValue`: `UpdateConfigurationValue` - persistence and re-merge
//! - `InspectValue`: `InspectConfigurationValue` - introspection across
//! scopes
//! - `Loading`: `ReadAndParseConfigurationFile`,
//! `InitializeAndMergeConfigurations`
//!
//! The single `impl ConfigurationProvider for MountainEnvironment` block in
//! this file delegates to those helper functions. This satisfies Rust's orphan
//! rules while keeping code organized.

use CommonLibrary::{
	Configuration::{
		ConfigurationInspector::ConfigurationInspector,
		ConfigurationProvider::ConfigurationProvider,
		DTO::{
			ConfigurationOverridesDTO::ConfigurationOverridesDTO,
			ConfigurationTarget::ConfigurationTarget,
			InspectResultDataDTO::InspectResultDataDTO,
		},
	},
	Error::CommonError::CommonError,
};
use async_trait::async_trait;

// Private helper modules (not re-exported)
mod GetValue;

mod UpdateValue;

mod InspectValue;

/// Configuration loading — parses, caches, and merges configuration files from
/// disk.
///
/// Exposed publicly for external callers like `ConfigurationInitialize`.
pub mod Loading; // Make public for external callers like ConfigurationInitialize

#[async_trait]
impl ConfigurationProvider for crate::Environment::MountainEnvironment::MountainEnvironment {
	async fn GetConfigurationValue(
		&self,

		Section:Option<String>,

		Overrides:ConfigurationOverridesDTO,
	) -> Result<serde_json::Value, CommonError> {
		GetValue::get_configuration_value(self, Section, Overrides).await
	}

	async fn UpdateConfigurationValue(
		&self,

		Key:String,

		Value:serde_json::Value,

		Target:ConfigurationTarget,

		Overrides:ConfigurationOverridesDTO,

		ScopeToLanguage:Option<bool>,
	) -> Result<(), CommonError> {
		UpdateValue::update_configuration_value(self, Key, Value, Target, Overrides, ScopeToLanguage).await
	}
}

#[async_trait]
impl ConfigurationInspector for crate::Environment::MountainEnvironment::MountainEnvironment {
	async fn InspectConfigurationValue(
		&self,

		Key:String,

		Overrides:ConfigurationOverridesDTO,
	) -> Result<Option<InspectResultDataDTO>, CommonError> {
		InspectValue::inspect_configuration_value(self, Key, Overrides).await
	}
}
