use std::sync::Arc;

use Common::{
	config::{
		ConfigInspector,
		ConfigProvider,
		dto::{ConfigurationOverridesDto, ConfigurationTarget, InspectResultDataDto},
	},
	environment::Requires,
	error::CommonError,
};
use async_trait::async_trait;
use serde_json::Value;

use crate::{environment::MountainEnvironment, handlers::config as ConfigHandler};

#[async_trait]
impl ConfigProvider for MountainEnvironment {
	async fn GetConfigurationValue(
		&self,
		Section:Option<String>,
		Overrides:ConfigurationOverridesDto,
	) -> Result<Value, CommonError> {
		ConfigHandler::GetConfigurationValueLogic(&self.AppHandle, Section, Overrides).await
	}

	async fn UpdateConfigurationValue(
		&self,
		Key:String,
		ValueToSet:Value,
		Target:ConfigurationTarget,
		Overrides:ConfigurationOverridesDto,
		ScopeToLanguage:Option<bool>,
	) -> Result<(), CommonError> {
		ConfigHandler::UpdateConfigurationValueLogic(
			&self.AppHandle,
			Key,
			ValueToSet,
			Target,
			Overrides,
			ScopeToLanguage,
		)
		.await
	}
}

#[async_trait]
impl ConfigInspector for MountainEnvironment {
	async fn InspectConfigurationValue(
		&self,
		Key:String,
		Overrides:ConfigurationOverridesDto,
	) -> Result<Option<InspectResultDataDto>, CommonError> {
		ConfigHandler::InspectConfigurationValueLogic(&self.AppHandle, Key, Overrides).await
	}
}

impl Requires<Arc<dyn ConfigProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn ConfigProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn ConfigInspector + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn ConfigInspector + Send + Sync> { Arc::new(self.clone()) }
}
