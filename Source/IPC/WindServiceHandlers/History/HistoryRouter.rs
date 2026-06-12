//! Navigation history command router.

use std::sync::Arc;

use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::Navigation::{
		HistoryCanGoBack::Fn as HistoryCanGoBack,
		HistoryCanGoForward::Fn as HistoryCanGoForward,
		HistoryClear::Fn as HistoryClear,
		HistoryGetStack::Fn as HistoryGetStack,
		HistoryGoBack::Fn as HistoryGoBack,
		HistoryGoForward::Fn as HistoryGoForward,
		HistoryPush::Fn as HistoryPush,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Routes history commands. Returns Some(result) for handled commands,
/// None otherwise.
pub(crate) async fn route(
	RunTime:Arc<ApplicationRunTime>,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		"history:goBack" => {
			dev_log!("history", "history:goBack");

			Some(HistoryGoBack(RunTime.clone()).await)
		},

		"history:goForward" => {
			dev_log!("history", "history:goForward");

			Some(HistoryGoForward(RunTime.clone()).await)
		},

		"history:canGoBack" => {
			dev_log!("history", "history:canGoBack");

			Some(HistoryCanGoBack(RunTime.clone()).await)
		},

		"history:canGoForward" => {
			dev_log!("history", "history:canGoForward");

			Some(HistoryCanGoForward(RunTime.clone()).await)
		},

		"history:push" => {
			dev_log!("history", "history:push");

			Some(HistoryPush(RunTime.clone(), Arguments).await)
		},

		"history:clear" => {
			dev_log!("history", "history:clear");

			Some(HistoryClear(RunTime.clone()).await)
		},

		"history:getStack" => {
			dev_log!("history", "history:getStack");

			Some(HistoryGetStack(RunTime.clone()).await)
		},

		_ => None,
	}
}
