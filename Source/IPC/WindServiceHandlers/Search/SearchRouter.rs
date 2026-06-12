//! Search command router - delegates all `search:*` IPC commands.

use std::sync::Arc;

use serde_json::Value;

use super::*;
use crate::{
	IPC::WindServiceHandlers::Search::{FindFiles::Fn as SearchFindFiles, FindInFiles::Fn as SearchFindInFiles},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Routes search commands. Returns Some(result) for handled commands, None
/// otherwise.
pub(crate) async fn route(
	RunTime:Arc<ApplicationRunTime>,

	_command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match _command {
		"search:findInFiles" | "search:textSearch" | "search:searchText" => {
			dev_log!("search", "search:findInFiles");

			Some(SearchFindInFiles(RunTime.clone(), Arguments).await)
		},

		"search:findFiles" | "search:fileSearch" | "search:searchFile" => {
			dev_log!("search", "search:findFiles");

			Some(SearchFindFiles(RunTime.clone(), Arguments).await)
		},

		"search:cancel" => {
			if let Some(SearchId) = Arguments.first().and_then(|V| V.as_u64()) {
				let Flags = &RunTime.Environment.ApplicationState.Feature.SearchCancellationFlags;

				if let Some(Flag) = Flags.get(&SearchId) {
					Flag.store(true, std::sync::atomic::Ordering::Relaxed);
				}

				let ActiveSearches = &RunTime.Environment.ApplicationState.Feature.ActiveSearches;

				if let Some((_, Handle)) = ActiveSearches.remove(&SearchId) {
					Handle.abort();

					dev_log!("search", "search:cancel aborted id={}", SearchId);
				} else {
					dev_log!("search", "search:cancel id={} not found (already done?)", SearchId);
				}
			} else {
				dev_log!("search", "search:cancel (no id, ignoring)");
			}

			Some(Ok(Value::Null))
		},

		"search:clearCache" | "search:onDidChangeResult" => {
			dev_log!("search", "{} (stub-ack)", _command);

			Some(Ok(Value::Null))
		},

		_ => None,
	}
}
