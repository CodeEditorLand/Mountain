#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{Environment::Requires::Requires, Search::SearchProvider::SearchProvider};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect};

pub fn CreateEffect<R:Runtime>(
	MethodName:&str,
	Parameters:Value,
) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"findFiles" | "findTextInFiles" => {
			let MethodNameOwned = MethodName.to_string();
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SearchProvider> = run_time.Environment.Require();
						let Args = if let Some(Object) = Parameters.as_object() {
							(
								Object.get("pattern").cloned().unwrap_or_default(),
								Object.get("options").cloned().unwrap_or_default(),
							)
						} else {
							(
								Parameters.get(0).cloned().unwrap_or_default(),
								Parameters.get(1).cloned().unwrap_or_default(),
							)
						};
						let (Pattern, Options) = Args;
						if MethodNameOwned == "findTextInFiles" {
							provider.TextSearch(Pattern, Options).await.map_err(|e| e.to_string())
						} else {
							Ok(json!([]))
						}
					})
				};
			Some(Ok(Box::new(effect)))
		},

		"Search.TextSearch" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn SearchProvider> = run_time.Environment.Require();
						let query = Parameters.get(0).cloned().unwrap_or_default();
						let options = Parameters.get(1).cloned().unwrap_or_default();
						provider.TextSearch(query, options).await.map_err(|e| e.to_string())
					})
				};
			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
