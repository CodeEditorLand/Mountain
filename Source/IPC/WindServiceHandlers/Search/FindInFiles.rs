//! Wire method: `search:findInFiles` / `search:textSearch`.
//! Delegates to `SearchProvider::TextSearch`.
//!
//! Each call mints a `search_id`, spawns the search as an abortable
//! `tokio` task, and stores the `AbortHandle` in
//! `ApplicationState.Feature.ActiveSearches` so that `search:cancel`
//! can abort the in-flight walk.

use std::sync::{Arc, atomic::Ordering as AtomicOrdering};

use serde_json::{Value, json};

use crate::{
	IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_bool,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Normalizes a positional include/exclude argument that arrives as either a
/// single glob string or a `string[]` into a JSON array of non-empty glob
/// strings. Returns `None` when nothing usable remains.
fn NormalizeGlobArgument(Argument:Option<&Value>) -> Option<Vec<Value>> {
	match Argument {
		Some(Value::String(Pattern)) if !Pattern.is_empty() => Some(vec![Value::String(Pattern.clone())]),

		Some(Value::Array(Items)) => {
			let Patterns:Vec<Value> = Items
				.iter()
				.filter(|Item| Item.as_str().map(|Pattern| !Pattern.is_empty()).unwrap_or(false))
				.cloned()
				.collect();

			if Patterns.is_empty() { None } else { Some(Patterns) }
		},

		_ => None,
	}
}

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, mut Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Search::SearchProvider::SearchProvider;

	let (QueryValue, mut OptionsValue) = if Arguments.first().map(|V| V.is_object()).unwrap_or(false) {
		let Query = Arguments.remove(0);

		let Options = Arguments.into_iter().next().unwrap_or(Value::Null);

		(Query, Options)
	} else if let Some(Pattern) = Arguments.first().and_then(|V| V.as_str()).map(str::to_owned) {
		let IsRegex = arg_bool(&Arguments, 1);

		let IsCase = arg_bool(&Arguments, 2);

		let IsWord = arg_bool(&Arguments, 3);

		// Positional wire shape (Wind's Effect SearchService):
		// [pattern, isRegex, isCaseSensitive, isWordMatch, include,
		// exclude, maxResults]. include/exclude accept both a single
		// glob string and a string[].
		let mut Options = serde_json::Map::new();

		if let Some(Include) = NormalizeGlobArgument(Arguments.get(4)) {
			Options.insert("include".to_string(), Value::Array(Include));
		}

		if let Some(Exclude) = NormalizeGlobArgument(Arguments.get(5)) {
			Options.insert("exclude".to_string(), Value::Array(Exclude));
		}

		if let Some(MaxResults) = Arguments.get(6).and_then(|V| V.as_u64()) {
			Options.insert("maxResults".to_string(), json!(MaxResults));
		}

		(
			json!({
				"pattern": Pattern,
				"isRegExp": IsRegex,
				"isCaseSensitive": IsCase,
				"isWordMatch": IsWord,
			}),
			Value::Object(Options),
		)
	} else {
		return Err("search:findInFiles requires pattern or TextSearchQuery".to_string());
	};

	// Mint a stable search_id for this call so `search:cancel` can
	// abort the in-flight task without a race against future searches.
	let SearchId = RunTime
		.Environment
		.ApplicationState
		.Feature
		.SearchIdCounter
		.fetch_add(1, AtomicOrdering::Relaxed);

	// Register the cooperative cancellation flag and thread the id into
	// the options payload (`__searchId`). The provider's synchronous
	// ripgrep walk polls the flag per entry - the task-level abort below
	// only lands at an await point, which the walk never reaches.
	let CancelFlag = Arc::new(std::sync::atomic::AtomicBool::new(false));

	let CancellationFlags = RunTime.Environment.ApplicationState.Feature.SearchCancellationFlags.clone();

	CancellationFlags.insert(SearchId, CancelFlag);

	if !OptionsValue.is_object() {
		OptionsValue = json!({});
	}

	if let Some(Object) = OptionsValue.as_object_mut() {
		Object.insert("__searchId".to_string(), json!(SearchId));
	}

	dev_log!(
		"search",
		"search:textSearch id={} delegating to SearchProvider::TextSearch",
		SearchId
	);

	// Clone what the spawned task needs before moving into the closure.
	let Environment = RunTime.Environment.clone();

	let ActiveSearches = RunTime.Environment.ApplicationState.Feature.ActiveSearches.clone();

	let Handle = tokio::task::spawn(async move { Environment.TextSearch(QueryValue, OptionsValue).await });

	let AbortHandle = Handle.abort_handle();

	ActiveSearches.insert(SearchId, AbortHandle);

	let Result = Handle.await;

	// Clean up regardless of outcome.
	ActiveSearches.remove(&SearchId);

	CancellationFlags.remove(&SearchId);

	match Result {
		Ok(Ok(Value)) => Ok(Value),

		Ok(Err(CommonError)) => Err(CommonError.to_string()),

		// Task was aborted via search:cancel - return empty results.
		Err(JoinError) if JoinError.is_cancelled() => {
			dev_log!("search", "search:textSearch id={} cancelled", SearchId);

			Ok(json!([]))
		},

		Err(JoinError) => Err(JoinError.to_string()),
	}
}
