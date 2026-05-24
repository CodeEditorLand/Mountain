use std::sync::Arc;

use CommonLibrary::{
	Configuration::{
		ConfigurationInspector::ConfigurationInspector,
		ConfigurationProvider::ConfigurationProvider,
		DTO::ConfigurationTarget::ConfigurationTarget,
	},
	Environment::Requires::Requires,
	IPC::IPCProvider::IPCProvider as IPCProviderTrait,
};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track::Effect::{
		CreateEffectForRequest::Utilities::Params::{StrObjOrPos, StringAt, U64At, ValAt},
		MappedEffectType::MappedEffect,
	},
	dev_log,
};

async fn UpdateConfigurationValueAndNotify(
	RunTime:Arc<ApplicationRunTime>,

	key:String,

	value:Value,

	target:ConfigurationTarget,

	log_prefix:&str,
) -> Result<Value, String> {
	use tauri::Emitter;

	let Provider:Arc<dyn ConfigurationProvider> = RunTime.Environment.Require();

	let KeyForEvents = key.clone();

	let result = provider
		.UpdateConfigurationValue(key, value, target, Default::default(), None)
		.await;

	if result.is_ok() {
		let Payload = json!({
			"keys": [KeyForEvents.clone()],
			"affected": [KeyForEvents.clone()],
		});

		let AppHandle = RunTime.Environment.ApplicationHandle.clone();

		if let Err(Error) = AppHandle.emit("sky://configuration/changed", Payload.clone()) {
			dev_log!(
				"config",
				"warn: [{}] sky://configuration/changed emit failed: {}",
				log_prefix,
				Error
			);
		}

		let IPCProvider:Arc<dyn IPCProviderTrait> = RunTime.Environment.Require();

		if let Err(Error) = IPCProvider
			.SendNotificationToSideCar("cocoon-main".to_string(), "configuration.change".to_string(), Payload)
			.await
		{
			dev_log!(
				"config",
				"warn: [{}] Cocoon configuration.change notification failed: {}",
				log_prefix,
				Error
			);
		}
	}

	result.map(|_| json!(null)).map_err(|E| e.to_string())
}

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"config.Get" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn ConfigurationInspector> = RunTime.Environment.Require();
				let Key = StrObjOrPos(&Parameters, "key", 0).to_string();
				let result = provider.InspectConfigurationValue(Key, Default::default()).await;
				result
					.map(|Inspection| serde_json::to_value(Inspection).unwrap_or(Value::Null))
					.map_err(|E| e.to_string())
			})
		},

		"config.update" => {
			crate::effect!(RunTime, {
				let (Key, Value_, Target) = if let Some(Object) = Parameters.as_object() {
					let K = Object.get("key").and_then(Value::as_str).unwrap_or("").to_string();
					let V = Object.get("value").cloned().unwrap_or_default();
					let T = match Object.get("target").and_then(Value::as_u64) {
						Some(0) => ConfigurationTarget::User,
						Some(1) => ConfigurationTarget::Workspace,
						_ => ConfigurationTarget::User,
					};
					(K, V, T)
				} else {
					let K = StringAt(&Parameters, 0);
					let V = ValAt(&Parameters, 1);
					let T = match U64At(&Parameters, 2) {
						0 => ConfigurationTarget::User,
						1 => ConfigurationTarget::Workspace,
						_ => ConfigurationTarget::User,
					};
					(K, V, T)
				};
				UpdateConfigurationValueAndNotify(RunTime, Key, Value_, Target, "config.update").await
			})
		},

		"Configuration.Inspect" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn ConfigurationInspector> = RunTime.Environment.Require();
				let section = StringAt(&Parameters, 0);
				let result = provider.InspectConfigurationValue(section, Default::default()).await;
				result
					.map(|Inspection| serde_json::to_value(Inspection).unwrap_or(Value::Null))
					.map_err(|E| e.to_string())
			})
		},

		"Configuration.Update" => {
			crate::effect!(RunTime, {
				let Key = StringAt(&Parameters, 0);
				let value = ValAt(&Parameters, 1);
				let Target = match U64At(&Parameters, 2) {
					0 => ConfigurationTarget::User,
					1 => ConfigurationTarget::Workspace,
					_ => ConfigurationTarget::User,
				};
				UpdateConfigurationValueAndNotify(RunTime, key, value, target, "Configuration.Update").await
			})
		},

		_ => None,
	}
}
