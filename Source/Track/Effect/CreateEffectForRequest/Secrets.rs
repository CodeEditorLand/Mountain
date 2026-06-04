pub fn Matches(MethodName:&str) -> bool {
	match MethodName {
		"secrets.get" | "secrets.store" | "secrets.delete" => true,

		_ => false,
	}
}

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Secret::SecretProvider::SecretProvider};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{string_at, string_at_or},
	MappedEffectType::MappedEffect,
};

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
		let Key = string_at(Parameters, 0);

		let ExtensionId = string_at_or(Parameters, 2, "unknown");

		(Key, ExtensionId)
	}
}

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"secrets.get" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn SecretProvider> = run_time.Environment.Require();

				let (Key, ExtensionId) = ExtractSecretKey(&Parameters);

				match provider.GetSecret(ExtensionId, Key).await {
					Ok(Some(Value)) => Ok(json!(Value)),
					Ok(None) => Ok(Value::Null),
					Err(Error) => Err(Error.to_string()),
				}
			})
		},

		"secrets.store" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn SecretProvider> = run_time.Environment.Require();

				let (Key, ExtensionId) = ExtractSecretKey(&Parameters);

				let SecretValue = if let Some(Object) = Parameters.as_object() {
					Object.get("value").and_then(Value::as_str).unwrap_or("").to_string()
				} else {
					string_at(&Parameters, 1)
				};

				provider
					.StoreSecret(ExtensionId, Key, SecretValue)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		"secrets.delete" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn SecretProvider> = run_time.Environment.Require();

				let (Key, ExtensionId) = ExtractSecretKey(&Parameters);

				provider
					.DeleteSecret(ExtensionId, Key)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		_ => None,
	}
}
