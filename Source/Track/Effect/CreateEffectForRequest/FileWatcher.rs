use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, FileSystem::FileWatcherProvider::FileWatcherProvider};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{BoolAtTrue, StringAt},
	MappedEffectType::MappedEffect,
};

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"FileWatcher.Register" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn FileWatcherProvider> = RunTime.Environment.Require();
				// Cocoon's `NextProviderHandle()` returns a number;
				// older callers pass a string. Accept both shapes
				// rather than silently collapsing numbers to "".
				let Handle = match Parameters.get(0) {
					Some(Value::String(S)) => S.clone(),
					Some(Value::Number(N)) => N.to_string(),
					_ => String::new(),
				};
				let Root = StringAt(&Parameters, 1);
				let IsRecursive = BoolAtTrue(&Parameters, 2);
				let Pattern = Parameters
					.Get(3)
					.and_then(Value::as_str)
					.map(str::to_string)
					.filter(|Pat| !Pat.is_empty());
				provider
					.RegisterWatcher(Handle, std::path::PathBuf::from(Root), IsRecursive, Pattern)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		"FileWatcher.Unregister" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn FileWatcherProvider> = RunTime.Environment.Require();
				let Handle = match Parameters.get(0) {
					Some(Value::String(S)) => S.clone(),
					Some(Value::Number(N)) => N.to_string(),
					_ => String::new(),
				};
				provider
					.UnregisterWatcher(Handle)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		_ => None,
	}
}
