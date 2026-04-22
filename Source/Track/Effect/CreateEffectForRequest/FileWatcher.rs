#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{Environment::Requires::Requires, FileSystem::FileWatcherProvider::FileWatcherProvider};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect};

pub fn CreateEffect<R:Runtime>(
	MethodName:&str,
	Parameters:Value,
) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"FileWatcher.Register" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn FileWatcherProvider> = run_time.Environment.Require();
						let Handle =
							Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let Root =
							Parameters.get(1).and_then(Value::as_str).unwrap_or("").to_string();
						let IsRecursive =
							Parameters.get(2).and_then(Value::as_bool).unwrap_or(true);
						let Pattern = Parameters
							.get(3)
							.and_then(Value::as_str)
							.map(str::to_string)
							.filter(|Pat| !Pat.is_empty());
						provider
							.RegisterWatcher(Handle, std::path::PathBuf::from(Root), IsRecursive, Pattern)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};
			Some(Ok(Box::new(effect)))
		},

		"FileWatcher.Unregister" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn FileWatcherProvider> = run_time.Environment.Require();
						let Handle =
							Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						provider
							.UnregisterWatcher(Handle)
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
