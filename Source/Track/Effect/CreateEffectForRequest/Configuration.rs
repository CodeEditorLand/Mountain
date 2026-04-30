#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

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

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect, dev_log};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"config.get" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn ConfigurationInspector> = run_time.Environment.Require();
						let Key = if let Some(Object) = Parameters.as_object() {
							Object.get("key").and_then(Value::as_str).unwrap_or("").to_string()
						} else {
							Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string()
						};
						let result = provider.InspectConfigurationValue(Key, Default::default()).await;
						result
							.map(|Inspection| serde_json::to_value(Inspection).unwrap_or(Value::Null))
							.map_err(|e| e.to_string())
					})
				};
			Some(Ok(Box::new(effect)))
		},

		"config.update" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						use tauri::Emitter;
						let provider:Arc<dyn ConfigurationProvider> = run_time.Environment.Require();
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
							let K = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
							let V = Parameters.get(1).cloned().unwrap_or_default();
							let T = match Parameters.get(2).and_then(Value::as_u64) {
								Some(0) => ConfigurationTarget::User,
								Some(1) => ConfigurationTarget::Workspace,
								_ => ConfigurationTarget::User,
							};
							(K, V, T)
						};
						let KeyForEvents = Key.clone();
						let result = provider
							.UpdateConfigurationValue(Key, Value_, Target, Default::default(), None)
							.await;
						if result.is_ok() {
							let Payload = json!({
								"keys": [KeyForEvents.clone()],
								"affected": [KeyForEvents.clone()],
							});
							let AppHandle = run_time.Environment.ApplicationHandle.clone();
							let _ = AppHandle.emit("sky://configuration/changed", Payload.clone());
							let IPCProvider:Arc<dyn IPCProviderTrait> = run_time.Environment.Require();
							let _ = IPCProvider
								.SendNotificationToSideCar(
									"cocoon-main".to_string(),
									"configuration.change".to_string(),
									Payload,
								)
								.await;
						}
						result.map(|_| json!(null)).map_err(|e| e.to_string())
					})
				};
			Some(Ok(Box::new(effect)))
		},

		"Configuration.Inspect" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn ConfigurationInspector> = run_time.Environment.Require();
						let section = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let result = provider.InspectConfigurationValue(section, Default::default()).await;
						result
							.map(|Inspection| serde_json::to_value(Inspection).unwrap_or(Value::Null))
							.map_err(|e| e.to_string())
					})
				};
			Some(Ok(Box::new(effect)))
		},

		"Configuration.Update" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						use tauri::Emitter;
						let provider:Arc<dyn ConfigurationProvider> = run_time.Environment.Require();
						let key = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let value = Parameters.get(1).cloned().unwrap_or_default();
						let target = match Parameters.get(2).and_then(Value::as_u64) {
							Some(0) => ConfigurationTarget::User,
							Some(1) => ConfigurationTarget::Workspace,
							_ => ConfigurationTarget::User,
						};
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
									"warn: [Configuration.Update] sky://configuration/changed emit failed: {}",
									Error
								);
							}
							let IPCProvider:Arc<dyn IPCProviderTrait> = run_time.Environment.Require();
							if let Err(Error) = IPCProvider
								.SendNotificationToSideCar(
									"cocoon-main".to_string(),
									"configuration.change".to_string(),
									Payload,
								)
								.await
							{
								dev_log!(
									"config",
									"warn: [Configuration.Update] Cocoon configuration.change notification failed: {}",
									Error
								);
							}
						}
						result.map(|_| json!(null)).map_err(|e| e.to_string())
					})
				};
			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
