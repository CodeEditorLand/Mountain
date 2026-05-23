use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	IPC::{DTO::ProxyTarget::ProxyTarget, IPCProvider::IPCProvider as IPCProviderTrait},
};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{
	Track::Effect::{CreateEffectForRequest::Utilities::Params::val_at, MappedEffectType::MappedEffect},
	dev_log,
};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Task.Fetch" => {
			crate::effect!(run_time, {
				let filter = val_at(&Parameters, 0);
				let IPCProvider:Arc<dyn IPCProviderTrait> = run_time.Environment.Require();
				let Method = format!("{}$fetchTasks", ProxyTarget::ExtHostTaskService.GetTargetPrefix());
				match IPCProvider
					.SendRequestToSideCar("cocoon-main".to_string(), Method, json!([filter]), 5000)
					.await
				{
					Ok(value) => Ok(value),
					Err(error) => {
						dev_log!("ipc", "warn: [Task.Fetch] extension did not answer ({:?}); returning []", error);
						Ok(json!([]))
					},
				}
			})
		},

		"Task.Execute" => {
			crate::effect!(run_time, {
				let task = val_at(&Parameters, 0);
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
		},

		_ => None,
	}
}
