pub fn Matches(MethodName:&str) -> bool {
	match MethodName {
		"config.get" | "config.update" | "Configuration.Inspect" | "Configuration.Update" => true,

		_ => false,
	}
}

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
		CreateEffectForRequest::Utilities::Params::{str_obj_or_pos, string_at, u64_at, val_at},
		MappedEffectType::MappedEffect,
	},
	dev_log,
};

async fn UpdateConfigurationValueAndNotify(
	run_time:Arc<ApplicationRunTime>,

	key:String,

	value:Value,

	target:ConfigurationTarget,

	log_prefix:&str,
) -> Result<Value, String> {
	use tauri::Emitter;

	let provider:Arc<dyn ConfigurationProvider> = run_time.Environment.Require();

	let KeyForEvents = key.clone();

	let result = provider
		.UpdateConfigurationValue(key, value, target, Default::default(), None)
		.await;

	if result.is_ok() {
		let Payload = json!({
			"keys": [KeyForEvents.clone()],
			"affected": [KeyForEvents.clone()],
		});

		let AppHandle = run_time.Environment.ApplicationHandle.clone();

		if let Err(Error) = AppHandle.emit("sky://configuration/changed", Payload.clone()) {
			dev_log!(
				"config",
				"warn: [{}] sky://configuration/changed emit failed: {}",
				log_prefix,
				Error
			);
		}

		let IPCProvider:Arc<dyn IPCProviderTrait> = run_time.Environment.Require();

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

	result.map(|_| json!(null)).map_err(|e| e.to_string())
}

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:&Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"config.get" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn ConfigurationInspector> = run_time.Environment.Require();

				let Key = str_obj_or_pos(&Parameters, "key", 0).to_string();

				let result = provider.InspectConfigurationValue(Key, Default::default()).await;

				result
					.map(|Inspection| serde_json::to_value(Inspection).unwrap_or(Value::Null))
					.map_err(|e| e.to_string())
			})
		},

		"config.update" => {
			crate::effect!(run_time, {
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
					let K = string_at(&Parameters, 0);

					let V = val_at(&Parameters, 1);

					let T = match u64_at(&Parameters, 2) {
						0 => ConfigurationTarget::User,
						1 => ConfigurationTarget::Workspace,
						_ => ConfigurationTarget::User,
					};

					(K, V, T)
				};

				UpdateConfigurationValueAndNotify(run_time, Key, Value_, Target, "config.update").await
			})
		},

		"Configuration.Inspect" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn ConfigurationInspector> = run_time.Environment.Require();

				let section = string_at(&Parameters, 0);

				let result = provider.InspectConfigurationValue(section, Default::default()).await;

				result
					.map(|Inspection| serde_json::to_value(Inspection).unwrap_or(Value::Null))
					.map_err(|e| e.to_string())
			})
		},

		"Configuration.Update" => {
			crate::effect!(run_time, {
				let key = string_at(&Parameters, 0);

				let value = val_at(&Parameters, 1);

				let target = match u64_at(&Parameters, 2) {
					0 => ConfigurationTarget::User,
					1 => ConfigurationTarget::Workspace,
					_ => ConfigurationTarget::User,
				};

				UpdateConfigurationValueAndNotify(run_time, key, value, target, "Configuration.Update").await
			})
		},

		_ => None,
	}
}
