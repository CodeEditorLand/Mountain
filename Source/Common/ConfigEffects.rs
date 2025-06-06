
// Defines traits and effects for interacting with application configuration.
// This includes getting, updating, and inspecting configuration values from
// various sources.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::{
	ConfigurationDto::{ConfigurationTarget, IConfigurationOverrides, InspectResultData},
	Effect::ActionEffect,
	Environment::{Environment, Requires},
	Errors::CommonError,
	Runtime::AppRuntimeTrait,
};

/// A trait for environments that can provide configuration values.
#[async_trait]
pub trait ConfigProvider: Environment {
	/// Retrieves a configuration value for a given section, with optional
	/// overrides.
	async fn GetConfigurationValue(
		&self,
		Section:Option<String>,
		Overrides:IConfigurationOverrides,
	) -> Result<Value, CommonError>;

	/// Updates a configuration value at a specific target scope.
	async fn UpdateConfigurationValue(
		&self,
		Key:String,
		ValueToSet:Value,
		Target:ConfigurationTarget,
		Overrides:IConfigurationOverrides,
		ScopeToLanguage:Option<bool>,
	) -> Result<(), CommonError>;
}

/// A trait for environments that can inspect the sources of configuration
/// values.
#[async_trait]
pub trait ConfigInspector: Environment {
	/// Inspects a configuration value to see its value from different scopes
	/// (user, workspace, etc.).
	async fn InspectConfigurationValue(
		&self,
		Key:String,
		Overrides:IConfigurationOverrides,
	) -> Result<Option<InspectResultData>, CommonError>;
}

/// Creates an effect to get a configuration value.
pub fn GetConfiguration<RuntimeAccessType>(
	Section:Option<String>,
	OverridesValue:Value,          // Serialized IConfigurationOverrides
	_ScopeToLanguage:Option<bool>, // Note: this seems specific to update, included for signature consistency
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Value>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn ConfigProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let SectionClone = Section.clone();
		let OverridesValueClone = OverridesValue.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn ConfigProvider> = Environment.require();
			let OverridesParsed:IConfigurationOverrides = serde_json::from_value(OverridesValueClone).map_err(|e| {
				CommonError::InvalidArg {
					ArgumentName:"OverridesValue".to_string(),
					Reason:format!("Failed to parse IConfigurationOverrides: {}", e),
				}
			})?;
			Provider.GetConfigurationValue(SectionClone, OverridesParsed).await
		})
	}))
}

/// Creates an effect to update a configuration value.
pub fn UpdateConfiguration<RuntimeAccessType>(
	Key:String,
	ValueToSet:Value,
	TargetAsU32:u32,
	OverridesValue:Value,
	ScopeToLanguage:Option<bool>,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, ()>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn ConfigProvider>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let KeyClone = Key.clone();
		let ValueToSetClone = ValueToSet.clone();
		let OverridesValueClone = OverridesValue.clone();
		let ScopeToLanguageClone = ScopeToLanguage;
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Provider:Arc<dyn ConfigProvider> = Environment.require();
			let TargetParsed:ConfigurationTarget = serde_json::from_value(Value::from(TargetAsU32)).map_err(|e| {
				CommonError::InvalidArg {
					ArgumentName:"Target".to_string(),
					Reason:format!("Failed to parse ConfigurationTarget from u32 {}: {}", TargetAsU32, e),
				}
			})?;
			let OverridesParsed:IConfigurationOverrides = serde_json::from_value(OverridesValueClone).map_err(|e| {
				CommonError::InvalidArg {
					ArgumentName:"OverridesValue".to_string(),
					Reason:format!("Failed to parse IConfigurationOverrides: {}", e),
				}
			})?;
			Provider
				.UpdateConfigurationValue(
					KeyClone,
					ValueToSetClone,
					TargetParsed,
					OverridesParsed,
					ScopeToLanguageClone,
				)
				.await
		})
	}))
}

/// Creates an effect to inspect a configuration value.
pub fn InspectConfigurationValue<RuntimeAccessType>(
	Key:String,
	OverridesValue:Value,
) -> ActionEffect<Arc<RuntimeAccessType>, CommonError, Option<InspectResultData>>
where
	RuntimeAccessType: AppRuntimeTrait<RuntimeAccessType::EnvironmentType> + Send + Sync + 'static,
	RuntimeAccessType::EnvironmentType: Requires<Arc<dyn ConfigInspector>>, {
	ActionEffect::New(Arc::new(move |Accessor:Arc<RuntimeAccessType>| {
		let KeyClone = Key.clone();
		let OverridesValueClone = OverridesValue.clone();
		Box::pin(async move {
			let Environment = Accessor.GetEnvironment();
			let Inspector:Arc<dyn ConfigInspector> = Environment.require();
			let OverridesParsed:IConfigurationOverrides = serde_json::from_value(OverridesValueClone).map_err(|e| {
				CommonError::InvalidArg {
					ArgumentName:"OverridesValue".to_string(),
					Reason:format!("Failed to parse IConfigurationOverrides: {}", e),
				}
			})?;
			Inspector.InspectConfigurationValue(KeyClone, OverridesParsed).await
		})
	}))
}
