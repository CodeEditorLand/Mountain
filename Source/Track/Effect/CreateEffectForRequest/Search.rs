
//! # Search Effect (CreateEffectForRequest)
//!
//! Effect constructors for workspace search RPC methods. Handles file and
//! text search by delegating to `SearchProvider` and `WorkspaceProvider`
//! traits on `MountainEnvironment`.
//!
//! ## Methods handled
//!
//! | Method | Description |
//! |---|---|
//! | `findFiles` | Glob-based file search using `ignore`-aware walker |
//! | `findTextInFiles` | Full-text search delegating to `SearchProvider::TextSearch` |
//! | `Search.TextSearch` | Alternative text search RPC (separate method name) |
//!
//! `findFiles` reuses `WorkspaceProvider::FindFilesInWorkspace` to get the
//! same `ignore`-aware glob walker used by `search:fileSearch`.

use std::{future::Future, pin::Pin, sync::Arc};

use serde_json::{Value, json};
use tauri::Runtime;
use CommonLibrary::{
	Environment::Requires::Requires,
	Search::SearchProvider::SearchProvider,
	Workspace::WorkspaceProvider::WorkspaceProvider,
};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"findFiles" | "findTextInFiles" => {
			let MethodNameOwned = MethodName.to_string();

			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let _workspace:Arc<dyn WorkspaceProvider> = run_time.Environment.Require();
						let provider:Arc<dyn SearchProvider> = run_time.Environment.Require();
						// Accept three call shapes:
						//   - `{pattern, options}` named form
						//   - `[pattern, options]` positional from `TryMountainThenNode` Cocoon path
						//   - `pattern` bare (legacy single-arg)
						let Args = if let Some(Object) = Parameters.as_object() {
							(
								Object.get("pattern").cloned().unwrap_or_default(),
								Object.get("options").cloned().unwrap_or_default(),
							)
						} else if Parameters.is_array() {
							(
								Parameters.get(0).cloned().unwrap_or_default(),
								Parameters.get(1).cloned().unwrap_or_default(),
							)
						} else {
							(Parameters.clone(), Value::Null)
						};
						let (Pattern, Options) = Args;
						if MethodNameOwned == "findTextInFiles" {
							return provider.TextSearch(Pattern, Options).await.map_err(|e| e.to_string());
						}
						// `findFiles` - delegate to
						// `WorkspaceProvider::FindFilesInWorkspace` so we
						// get the same `ignore`-aware glob walker that
						// `search:fileSearch` uses. The trait returns
						// `Vec<Url>`; map to `Vec<String>` for the wire.
						if Pattern.is_null() {
							return Ok(json!([]));
						}
						let Exclude = Options.get("exclude").cloned().filter(|V| !V.is_null());
						let MaxResults = Options.get("maxResults").and_then(Value::as_u64).map(|N| N as usize);
						let UseIgnoreFiles = Options.get("useIgnoreFiles").and_then(Value::as_bool).unwrap_or(true);
						let FollowSymlinks = Options.get("followSymlinks").and_then(Value::as_bool).unwrap_or(false);
						let Urls = run_time
							.Environment
							.FindFilesInWorkspace(Pattern, Exclude, MaxResults, UseIgnoreFiles, FollowSymlinks)
							.await
							.map_err(|Error| Error.to_string())?;
						Ok(json!(Urls.into_iter().map(|U| U.to_string()).collect::<Vec<_>>()))
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
