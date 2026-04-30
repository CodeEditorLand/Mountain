#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{
	Environment::Requires::Requires,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider as IPCProviderTrait},
};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect, dev_log};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Task.Fetch" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let filter = Parameters.get(0).cloned().unwrap_or(Value::Null);
						let IPCProvider:Arc<dyn IPCProviderTrait> = run_time.Environment.Require();
						let Method = format!("{}$fetchTasks", ProxyTarget::ExtHostTaskService.GetTargetPrefix());
						match IPCProvider
							.SendRequestToSideCar("cocoon-main".to_string(), Method, json!([filter]), 5000)
							.await
						{
							Ok(value) => Ok(value),
							Err(error) => {
								dev_log!(
									"ipc",
									"warn: [Task.Fetch] extension did not answer ({:?}); returning []",
									error
								);
								Ok(json!([]))
							},
						}
					})
				};
			Some(Ok(Box::new(effect)))
		},

		"Task.Execute" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let task = Parameters.get(0).cloned().unwrap_or(Value::Null);
						let IPCProvider:Arc<dyn IPCProviderTrait> = run_time.Environment.Require();
						let Method = format!("{}$executeTask", ProxyTarget::ExtHostTaskService.GetTargetPrefix());
						match IPCProvider
							.SendRequestToSideCar("cocoon-main".to_string(), Method, json!([task]), 30000)
							.await
						{
							Ok(value) => Ok(value),
							Err(error) => {
								dev_log!(
									"ipc",
									"warn: [Task.Execute] extension did not answer ({:?}); returning null",
									error
								);
								Ok(json!(null))
							},
						}
					})
				};
			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
