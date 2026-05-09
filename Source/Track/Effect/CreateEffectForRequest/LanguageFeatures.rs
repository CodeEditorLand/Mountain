#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{
	Environment::Requires::Requires,
	LanguageFeature::{
		DTO::ProviderType::ProviderType,
		LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	},
};

use serde_json::{Value, json};

use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {

	match MethodName {

		"$languageFeatures:registerProvider" => {

			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {

					Box::pin(async move {
						let provider:Arc<dyn LanguageFeatureProviderRegistry> = run_time.Environment.Require();
						let id = Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let selector = Parameters.get(1).cloned().unwrap_or_default();
						let extension_id = Parameters.get(2).cloned().unwrap_or_default();
						let options = Parameters.get(3).cloned();
						provider
							.RegisterProvider(id, ProviderType::Hover, selector, extension_id, options)
							.await
							.map(|handle| json!(handle))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
