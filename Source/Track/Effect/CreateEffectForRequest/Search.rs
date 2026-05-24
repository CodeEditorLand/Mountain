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

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::Runtime;
use CommonLibrary::{
	Environment::Requires::Requires,
	Search::SearchProvider::SearchProvider,
	Workspace::WorkspaceProvider::WorkspaceProvider,
};

use crate::Track::Effect::{CreateEffectForRequest::Utilities::Params::ValAt, MappedEffectType::MappedEffect};

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"findFiles" | "findTextInFiles" => {
			let MethodNameOwned = MethodName.to_string();

			crate::effect!(RunTime, {
				let _workspace:Arc<dyn WorkspaceProvider> = RunTime.Environment.Require();
				let Provider:Arc<dyn SearchProvider> = RunTime.Environment.Require();
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
					(ValAt(&Parameters, 0), ValAt(&Parameters, 1))
				} else {
					(Parameters.clone(), Value::Null)
				};
				let (Pattern, Options) = Args;
				if MethodNameOwned == "findTextInFiles" {
					return provider.TextSearch(Pattern, Options).await.map_err(|E| e.to_string());
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
				let Urls = RunTime
					.Environment
					.FindFilesInWorkspace(Pattern, Exclude, MaxResults, UseIgnoreFiles, FollowSymlinks)
					.await
					.map_err(|Error| Error.to_string())?;
				Ok(json!(Urls.into_iter().map(|U| U.to_string()).collect::<Vec<_>>()))
			})
		},

		"Search.TextSearch" => {
			crate::effect!(RunTime, {
				let Provider:Arc<dyn SearchProvider> = RunTime.Environment.Require();
				let Query = ValAt(&Parameters, 0);
				let Options = ValAt(&Parameters, 1);
				provider.TextSearch(query, options).await.map_err(|E| e.to_string())
			})
		},

		_ => None,
	}
}
