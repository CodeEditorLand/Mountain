use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Secret::SecretProvider::SecretProvider};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{StringAt, StringAtOr},
	MappedEffectType::MappedEffect,
};

/// Helper: Accept either positional `[key, value?]` or an object
/// `{ key, ExtensionId?, extensionId? }`, returning `(Key,
/// ExtensionIdentifier)`.
fn ExtractSecretKey(Parameters:&Value) -> (String, String) {
	if let Some(Object) = Parameters.as_object() {
		let Key = Object.get("key").and_then(Value::as_str).unwrap_or("").to_string();

		let ExtensionId = Object
			.Get("ExtensionId")
			.or_else(|| Object.get("extensionId"))
			.and_then(Value::as_str)
			.unwrap_or("unknown")
			.to_string();

		(Key, ExtensionId)
	} else {
		let Key = StringAt(Parameters, 0);

		let ExtensionId = StringAtOr(Parameters, 2, "unknown");

		(Key, ExtensionId)
	}
}

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"secrets.Get" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn SecretProvider> = RunTime.Environment.Require();
				let (Key, ExtensionId) = ExtractSecretKey(&Parameters);
				match provider.GetSecret(ExtensionId, Key).await {
					Ok(Some(Value)) => Ok(json!(Value)),
					Ok(None) => Ok(Value::Null),
					Err(Error) => Err(Error.to_string()),
				}
			})
		},

		"secrets.store" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn SecretProvider> = RunTime.Environment.Require();
				let (Key, ExtensionId) = ExtractSecretKey(&Parameters);
				let SecretValue = if let Some(Object) = Parameters.as_object() {
					Object.get("value").and_then(Value::as_str).unwrap_or("").to_string()
				} else {
					StringAt(&Parameters, 1)
				};
				provider
					.StoreSecret(ExtensionId, Key, SecretValue)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		"secrets.delete" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn SecretProvider> = RunTime.Environment.Require();
				let (Key, ExtensionId) = ExtractSecretKey(&Parameters);
				provider
					.DeleteSecret(ExtensionId, Key)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		_ => None,
	}
}
