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
		"Authentication.GetSession" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider_id = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let scopes = Parameters.get(1).cloned().unwrap_or(json!([]));
						let options = Parameters.get(2).cloned().unwrap_or(json!({}));
						let IPCProvider:Arc<dyn IPCProviderTrait> = run_time.Environment.Require();
						let Method = format!("{}$getSession", ProxyTarget::ExtHostAuthentication.GetTargetPrefix());
						match IPCProvider
							.SendRequestToSideCar(
								"cocoon-main".to_string(),
								Method,
								json!([provider_id, scopes, options]),
								5000,
							)
							.await
						{
							Ok(value) => Ok(value),
							Err(error) => {
								dev_log!(
									"ipc",
									"warn: [Authentication.GetSession] extension did not answer ({:?}); returning null",
									error
								);
								Ok(json!(null))
							},
						}
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"Authentication.GetAccounts" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider_id = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let IPCProvider:Arc<dyn IPCProviderTrait> = run_time.Environment.Require();
						let Method = format!("{}$getAccounts", ProxyTarget::ExtHostAuthentication.GetTargetPrefix());
						match IPCProvider
							.SendRequestToSideCar("cocoon-main".to_string(), Method, json!([provider_id]), 5000)
							.await
						{
							Ok(value) => Ok(value),
							Err(error) => {
								dev_log!(
									"ipc",
									"warn: [Authentication.GetAccounts] extension did not answer ({:?}); returning []",
									error
								);
								Ok(json!([]))
							},
						}
					})
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
