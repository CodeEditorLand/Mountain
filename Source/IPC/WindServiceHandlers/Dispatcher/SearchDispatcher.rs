//! Search command dispatcher.

use serde_json::Value;

<<<<<<< HEAD
use crate::Search::{FindFiles::Fn as SearchFindFiles, FindInFiles::Fn as SearchFindInFiles};
=======
use crate::IPC::WindServiceHandlers::Search::{FindFiles::Fn as SearchFindFiles, FindInFiles::Fn as SearchFindInFiles};
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867

/// Dispatches search commands.
///
/// Handled commands:
/// - `search:findInFiles` / `search:textSearch` / `search:searchText`
/// - `search:findFiles` / `search:fileSearch` / `search:searchFile`
/// - `search:cancel` - aborts an in-flight search task by search_id
/// - `search:clearCache` / `search:onDidChangeResult` (no-op acks)
pub async fn dispatch_search(
<<<<<<< HEAD
	runtime:&crate::RunTime::ApplicationRunTime::ApplicationRunTime,
=======
	runtime:std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"search:findInFiles" | "search:textSearch" | "search:searchText" => {
			SearchFindInFiles(runtime.clone(), arguments).await
		},

		"search:findFiles" | "search:fileSearch" | "search:searchFile" => {
			SearchFindFiles(runtime.clone(), arguments).await
		},

<<<<<<< HEAD
		"search:cancel" => {
			// VS Code sends the search_id as arg[0] (number).
			// Look it up in the active-search map and abort the task.
			if let Some(SearchId) = arguments.first().and_then(|V| V.as_u64()) {
				let ActiveSearches = &runtime.Environment.ApplicationState.Feature.ActiveSearches;

				if let Some((_, Handle)) = ActiveSearches.remove(&SearchId) {
					Handle.abort();

					crate::dev_log!("search", "search:cancel aborted id={}", SearchId);
				} else {
					crate::dev_log!("search", "search:cancel id={} not found (already done?)", SearchId);
				}
			} else {
				crate::dev_log!("search", "search:cancel (no id, ignoring)");
			}

			Ok(Value::Null)
		},

		"search:clearCache" | "search:onDidChangeResult" => {
			crate::dev_log!("search", "{} (stub-ack)", command);

			Ok(Value::Null)
		},

=======
		"search:cancel" | "search:clearCache" | "search:onDidChangeResult" => {
			crate::dev_log!("search", "{} (stub-ack)", command);

			Ok(Value::Null)
		},

>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867
		_ => Err(format!("Unknown search command: {}", command)),
	}
}
