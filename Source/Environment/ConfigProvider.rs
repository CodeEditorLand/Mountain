// File: Environment/ConfigProvider.rs
// Implements the `ConfigProvider` and `ConfigInspector` traits for the
// `MountainEnvironment`. This file connects abstract configuration effects to
// the concrete logic in the handlers.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{
	ConfigEffect::{ConfigInspector, ConfigProvider, ConfigurationTarget, IConfigurationOverrides, InspectResultData},
	Environment::Requires,
	Errors::CommonError,
};
use async_trait::async_trait;
use log::{debug, info, trace, warn};
use serde_json::Value;

use crate::{
	AppState::AppState,
	Environment::{MountainEnvironment, Utils::MapAppStateLockErrorToCommonError},
	Handlers,
};

#[async_trait]
impl ConfigProvider for MountainEnvironment {
	/// Retrieves a configuration value.
	async fn GetConfigurationValue(
		&self,
		SectionKeyOption:Option<String>,
		Overrides:IConfigurationOverrides,
	) -> Result<Value, CommonError> {
		trace!(
			"[Environment ConfigProvider] GetConfigurationValue: Section='{:?}', Overrides.Resource='{:?}'",
			SectionKeyOption,
			Overrides.Resource.as_ref().and_then(|v| v.get("external"))
		);

		let AppStateInstance = self.GetAppState();
		let ConfigStateGuard = AppStateInstance
			.Configuration
			.lock()
			.map_err(MapAppStateLockErrorToCommonError)?;

		if Overrides.Resource.is_some() || Overrides.OverrideIdentifier.is_some() {
			warn!(
				"[Environment ConfigProvider] Overrides provided, but current implementation uses the pre-merged \
				 state. Fine-grained override resolution is limited."
			);
		}

		let ValueResult = ConfigStateGuard.GetValue(SectionKeyOption.as_deref(), Overrides.Resource.as_ref());
		debug!(
			"[Environment ConfigProvider] Value for section {:?}: (sample) {}...",
			SectionKeyOption,
			ValueResult.to_string().chars().take(70).collect::<String>()
		);
		Ok(ValueResult)
	}

	/// Updates a configuration value.
	async fn UpdateConfigurationValue(
		&self,
		KeyToUpdate:String,
		ValueToSet:Value,
		TargetScope:ConfigurationTarget,
		Overrides:IConfigurationOverrides,
		ScopeToLanguageOverride:Option<bool>,
	) -> Result<(), CommonError> {
		info!(
			"[Environment ConfigProvider] UpdateConfigurationValue: Key='{}', TargetScope={:?}",
			KeyToUpdate, TargetScope
		);

		let AppStateInstance = self.GetAppState();
		// The core logic is delegated to the configuration handler.
		let TargetConfigFilePath = Handlers::Config::GetConfigPathForTarget(
			&self.AppHandle,
			&AppStateInstance,
			TargetScope,
			&Overrides,
			ScopeToLanguageOverride.unwrap_or(false),
		)?;
		info!(
			"[Environment ConfigProvider] Target config file for update: {}",
			TargetConfigFilePath.display()
		);

		let mut CurrentTargetFileJsonContent =
			Handlers::Config::LoadJsonFileIfExistsOrDefault(&TargetConfigFilePath).await?;

		let mut EffectiveJsonNodeToUpdateIn = &mut CurrentTargetFileJsonContent;
		if ScopeToLanguageOverride.unwrap_or(false) {
			if let Some(LanguageIdentifierString) = &Overrides.OverrideIdentifier {
				let LanguageScopeKey = format!("[{}]", LanguageIdentifierString);
				if !EffectiveJsonNodeToUpdateIn.is_object() {
					*EffectiveJsonNodeToUpdateIn = serde_json::json!({});
				}
				EffectiveJsonNodeToUpdateIn = EffectiveJsonNodeToUpdateIn
					.as_object_mut()
					.unwrap()
					.entry(LanguageScopeKey)
					.or_insert_with(|| serde_json::json!({}));
			} else {
				warn!(
					"[Environment ConfigProvider] 'scopeToLanguage' is true but no languageId provided. Updating \
					 top-level of '{}'.",
					TargetConfigFilePath.display()
				);
			}
		}

		Handlers::Config::UpdateJsonValueAtPath(EffectiveJsonNodeToUpdateIn, &KeyToUpdate, ValueToSet);
		Handlers::Config::WriteJsonFile(&TargetConfigFilePath, CurrentTargetFileJsonContent).await?;

		let NewMergedConfigState =
			Handlers::Config::LoadAndMergeConfigurationsInternal(&self.AppHandle, &AppStateInstance).await?;
		AppStateInstance
			.Configuration
			.lock()
			.map_err(MapAppStateLockErrorToCommonError)?
			.Update(NewMergedConfigState);

		Handlers::Config::NotifyConfigChangedForKeys(&self.AppHandle, vec![KeyToUpdate]).await;
		Ok(())
	}
}

#[async_trait]
impl ConfigInspector for MountainEnvironment {
	/// Inspects a configuration value to see its different values across
	/// scopes.
	async fn InspectConfigurationValue(
		&self,
		Key:String,
		Overrides:IConfigurationOverrides,
	) -> Result<Option<InspectResultData>, CommonError> {
		info!(
			"[Environment ConfigInspector] Inspecting: Key='{}', Overrides.Resource='{:?}'",
			Key,
			Overrides.Resource.as_ref().and_then(|v| v.get("external"))
		);

		let AppStateInstance = self.GetAppState();
		let ConfigGuard = AppStateInstance
			.Configuration
			.lock()
			.map_err(MapAppStateLockErrorToCommonError)?;
		let EffectiveValue = ConfigGuard.GetValue(Some(&Key), Overrides.Resource.as_ref());

		if EffectiveValue.is_null() && !ConfigGuard.Data.get(&Key).is_some() {
			debug!(
				"[Environment ConfigInspector] Key '{}' not found in effective configuration.",
				Key
			);
			Ok(None)
		} else {
			warn!(
				"[Environment ConfigInspector] STUB: `inspect_configuration_value` only returns the effective value. \
				 Detailed scope inspection is not yet implemented."
			);
			Ok(Some(InspectResultData {
				EffectiveValue:Some(EffectiveValue),
				..Default::default()
			}))
		}
	}
}

impl Requires<Arc<dyn ConfigProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn ConfigProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn ConfigInspector + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn ConfigInspector + Send + Sync> { Arc::new(self.clone()) }
}
