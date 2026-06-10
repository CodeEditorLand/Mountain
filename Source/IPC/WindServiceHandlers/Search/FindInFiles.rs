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

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, mut Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::Search::SearchProvider::SearchProvider;

	let QueryValue = if Arguments.first().map(|V| V.is_object()).unwrap_or(false) {
		Arguments.remove(0)
	} else if let Some(Pattern) = Arguments.first().and_then(|V| V.as_str()) {
		let IsRegex = arg_bool(&Arguments, 1);

		let IsCase = arg_bool(&Arguments, 2);

		let IsWord = arg_bool(&Arguments, 3);

		json!({
			"pattern": Pattern,
			"isRegex": IsRegex,
			"isCaseSensitive": IsCase,
			"isWordMatch": IsWord,
		})
	} else {
		return Err("search:findInFiles requires pattern or TextSearchQuery".to_string());
	};

	let OptionsValue = Arguments.into_iter().next().unwrap_or(Value::Null);

	// Mint a stable search_id for this call so `search:cancel` can
	// abort the in-flight task without a race against future searches.
	let SearchId = RunTime
		.Environment
		.ApplicationState
		.Feature
		.SearchIdCounter
		.fetch_add(1, AtomicOrdering::Relaxed);

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
