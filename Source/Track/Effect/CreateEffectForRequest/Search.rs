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
							return provider.TextSearch(Pattern, Options).await.map_err(|e| e.to_string());
						}
						// Atom T3: route through the same glob-walker that
						// powers `search:findFiles` via the Wind service IPC.
						// Previously `findFiles` returned `[]` synthetically -
						// extensions calling `vscode.workspace.findFiles(…)`
						// got nothing even when matches existed on disk. The
						// Wind handler takes `[pattern, maxResults]`; map the
						// canonical vscode arg shape onto it.
						let PatternString = Pattern.as_str().map(str::to_string).unwrap_or_default();
						if PatternString.is_empty() {
							return Ok(json!([]));
						}
						let MaxResults = Options
							.get("maxResults")
							.and_then(Value::as_u64)
							.or_else(|| Options.as_u64())
							.unwrap_or(500);
						let Args = vec![json!(PatternString), json!(MaxResults)];
						crate::IPC::WindServiceHandlers::Search::handle_search_find_files(run_time.clone(), Args).await
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
