use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{Environment::Requires::Requires, Secret::SecretProvider::SecretProvider};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect};

/// Helper: Accept either positional `[key, value?]` or an object
/// `{ key, extension_id?, extensionId? }`, returning `(Key,
/// ExtensionIdentifier)`.
fn ExtractSecretKey(Parameters:&Value) -> (String, String) {
	if let Some(Object) = Parameters.as_object() {
		let Key = Object.get("key").and_then(Value::as_str).unwrap_or("").to_string();

		let ExtensionId = Object
			.get("extension_id")
			.or_else(|| Object.get("extensionId"))
			.and_then(Value::as_str)
			.unwrap_or("unknown")
			.to_string();

		(Key, ExtensionId)
	} else {
		let Key = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();

		let ExtensionId = Parameters.get(2).and_then(Value::as_str).unwrap_or("unknown").to_string();

		(Key, ExtensionId)
	}
}

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"secrets.get" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SecretProvider> = run_time.Environment.Require();
						let (Key, ExtensionId) = ExtractSecretKey(&Parameters);
						match provider.GetSecret(ExtensionId, Key).await {
							Ok(Some(Value)) => Ok(json!(Value)),
							Ok(None) => Ok(Value::Null),
							Err(Error) => Err(Error.to_string()),
						}
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"secrets.store" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SecretProvider> = run_time.Environment.Require();
						let (Key, ExtensionId) = ExtractSecretKey(&Parameters);
						let SecretValue = if let Some(Object) = Parameters.as_object() {
							Object.get("value").and_then(Value::as_str).unwrap_or("").to_string()
						} else {
							Parameters.get(1).and_then(Value::as_str).unwrap_or("").to_string()
						};
						provider
							.StoreSecret(ExtensionId, Key, SecretValue)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"secrets.delete" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SecretProvider> = run_time.Environment.Require();
						let (Key, ExtensionId) = ExtractSecretKey(&Parameters);
						provider
							.DeleteSecret(ExtensionId, Key)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
