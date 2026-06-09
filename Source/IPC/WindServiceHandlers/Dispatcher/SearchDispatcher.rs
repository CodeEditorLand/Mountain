//! Search command dispatcher.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::Search::{FindFiles::Fn as SearchFindFiles, FindInFiles::Fn as SearchFindInFiles};

/// Dispatches search commands.
///
/// Handled commands:
/// - `search:findInFiles` / `search:textSearch` / `search:searchText`
/// - `search:findFiles` / `search:fileSearch` / `search:searchFile`
/// - `search:cancel` / `search:clearCache` / `search:onDidChangeResult` (stubs)
pub async fn dispatch_search(
	runtime:std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,

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

		"search:cancel" | "search:clearCache" | "search:onDidChangeResult" => {
			crate::dev_log!("search", "{} (stub-ack)", command);

			Ok(Value::Null)
		},

		_ => Err(format!("Unknown search command: {}", command)),
	}
}
