#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Search handlers - find in files, find files by glob.
//!
//! **Both handlers now delegate to the properly-implemented trait
//! methods on `MountainEnvironment`** instead of carrying their own
//! inline fs-walk. The inline versions used naive `starts_with('.')`
//! hidden-file skipping (doesn't honour `.gitignore`), no regex engine,
//! a bogus `format!("file://{}", path)` URI constructor, and a single-
//! threaded walker. The trait impls live in:
//!
//! - `Environment/SearchProvider.rs` (`TextSearch`) - `grep-searcher` +
//!   `RegexMatcherBuilder` + `ignore::WalkBuilder::build_parallel()` with
//!   `PerFileSink` collection.
//! - `Environment/WorkspaceProvider.rs` (`FindFilesInWorkspace`) -
//!   `ignore`-aware glob walker with `.gitignore` support, max-result cap,
//!   symlink handling, and proper `Url::from_file_path` URI construction.
//!
//! This wiring was the "lot of dead code that needs to be connected"
//! the user flagged - the trait impls were reachable only through
//! `Environment.Require<dyn SearchProvider>()` / `WorkspaceProvider`
//! calls and no IPC handler ever issued those calls.

use std::sync::Arc;

use serde_json::{Value, json};
use CommonLibrary::{Search::SearchProvider::SearchProvider, Workspace::WorkspaceProvider::WorkspaceProvider};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

/// `search:findInFiles` / `search:textSearch` / `search:searchText`.
///
/// Wire contract (VS Code's `ProxyChannel.toService(search)` path):
/// positional args = [TextSearchQuery, TextSearchOptions]. The trait
/// method `SearchProvider::TextSearch` accepts the raw JSON and does
/// its own `serde_json::from_value::<TextSearchQuery>` so callers can
/// keep sending arbitrary shapes - we pass through directly.
pub async fn SearchFindInFiles(
	runtime:Arc<ApplicationRunTime>,
	mut args:Vec<Value>,
) -> Result<Value, String> {
	// Positional → named translation. VS Code's SearchService sends the
	// query object in slot 0; older Wind Effect callers passed flat
	// positional args (pattern, isRegex, isCase, isWord, include,
	// exclude, maxResults). Accept both by promoting flat args into a
	// TextSearchQuery-shaped object.
	let QueryValue = if args.first().map(|V| V.is_object()).unwrap_or(false) {
		args.remove(0)
	} else if let Some(Pattern) = args.first().and_then(|V| V.as_str()) {
		let IsRegex = args.get(1).and_then(|V| V.as_bool()).unwrap_or(false);
		let IsCase = args.get(2).and_then(|V| V.as_bool()).unwrap_or(false);
		let IsWord = args.get(3).and_then(|V| V.as_bool()).unwrap_or(false);
		json!({
			"pattern": Pattern,
			"isRegex": IsRegex,
			"isCaseSensitive": IsCase,
			"isWordMatch": IsWord,
		})
	} else {
		return Err("search:findInFiles requires pattern or TextSearchQuery".to_string());
	};

	let OptionsValue = args.into_iter().next().unwrap_or(Value::Null);

	dev_log!("search", "search:textSearch delegating to SearchProvider::TextSearch");

	runtime
		.Environment
		.TextSearch(QueryValue, OptionsValue)
		.await
		.map_err(|Error| Error.to_string())
}

/// `search:findFiles` / `search:fileSearch` / `search:searchFile`.
///
/// Wire contract: positional args = [includePattern, excludePattern?,
/// maxResults?, useIgnoreFiles?, followSymlinks?]. Delegates to
/// `WorkspaceProvider::FindFilesInWorkspace` which returns `Vec<Url>`;
/// we reshape to `Vec<String>` for the renderer.
pub async fn SearchFindFiles(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let IncludePattern = args
		.first()
		.cloned()
		.ok_or_else(|| "search:findFiles requires include pattern in slot 0".to_string())?;
	let ExcludePattern = args.get(1).cloned().filter(|V| !V.is_null());
	let MaxResults = args.get(2).and_then(|V| V.as_u64()).map(|N| N as usize);
	let UseIgnoreFiles = args.get(3).and_then(|V| V.as_bool()).unwrap_or(true);
	let FollowSymlinks = args.get(4).and_then(|V| V.as_bool()).unwrap_or(false);

	dev_log!(
		"search",
		"search:fileSearch delegating to WorkspaceProvider::FindFilesInWorkspace (ignore={}, symlinks={})",
		UseIgnoreFiles,
		FollowSymlinks
	);

	let Urls = runtime
		.Environment
		.FindFilesInWorkspace(IncludePattern, ExcludePattern, MaxResults, UseIgnoreFiles, FollowSymlinks)
		.await
		.map_err(|Error| Error.to_string())?;

	Ok(json!(Urls.into_iter().map(|U| U.to_string()).collect::<Vec<_>>()))
}
