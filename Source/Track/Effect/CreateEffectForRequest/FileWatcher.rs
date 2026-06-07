pub fn Matches(MethodName:&str) -> bool {

	match MethodName {
		"FileWatcher.Watch" | "FileWatcher.Unwatch" | "FileWatcher.WatchStatus" => true,

		_ => false,
	}
}

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, FileSystem::FileWatcherProvider::FileWatcherProvider};

use serde_json::{Value, json};

use tauri::Runtime;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{bool_at_true, string_at},
	MappedEffectType::MappedEffect,
};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {

	match MethodName {
		"FileWatcher.Register" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn FileWatcherProvider> = run_time.Environment.Require();

				// Cocoon's `NextProviderHandle()` returns a number;
				// older callers pass a string. Accept both shapes
				// rather than silently collapsing numbers to "".
				let Handle = match Parameters.get(0) {
					Some(Value::String(S)) => S.clone(),
					Some(Value::Number(N)) => N.to_string(),
					_ => String::new(),
				};

				let Root = string_at(&Parameters, 1);

				let IsRecursive = bool_at_true(&Parameters, 2);

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
		},

		"FileWatcher.Unregister" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn FileWatcherProvider> = run_time.Environment.Require();

				let Handle = match Parameters.get(0) {
					Some(Value::String(S)) => S.clone(),
					Some(Value::Number(N)) => N.to_string(),
					_ => String::new(),
				};

				provider
					.UnregisterWatcher(Handle)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		_ => None,
	}
}
