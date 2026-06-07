//! Navigation command dispatcher.

use crate::Navigation::{
    HistoryCanGoBack::Fn as HistoryCanGoBack,
    HistoryCanGoForward::Fn as HistoryCanGoForward,
    HistoryClear::Fn as HistoryClear,
    HistoryGetStack::Fn as HistoryGetStack,
    HistoryGoBack::Fn as HistoryGoBack,
    HistoryGoForward::Fn as HistoryGoForward,
    HistoryPush::Fn as HistoryPush,
    LabelGetBase::Fn as LabelGetBase,
    LabelGetURI::Fn as LabelGetURI,
    LabelGetWorkspace::Fn as LabelGetWorkspace,
};

use serde_json::Value;

/// Dispatches navigation commands.
///
/// Handled commands:
/// - `history:goBack`
/// - `history:goForward`
/// - `history:canGoBack`
/// - `history:canGoForward`
/// - `history:push`
/// - `history:clear`
/// - `history:getStack`
/// - `label:getUri`
/// - `label:getWorkspace`
/// - `label:getBase`
pub async fn dispatch_navigation(
    runtime: &crate::RunTime::ApplicationRunTime::ApplicationRunTime,

    command: &str,

    arguments: Vec<Value>,
) -> Result<Value, String> {

    match command {
        "history:goBack" => HistoryGoBack(runtime.clone()).await,

        "history:goForward" => HistoryGoForward(runtime.clone()).await,

        "history:canGoBack" => HistoryCanGoBack(runtime.clone()).await,

        "history:canGoForward" => HistoryCanGoForward(runtime.clone()).await,

        "history:push" => HistoryPush(runtime.clone(), arguments).await,

        "history:clear" => HistoryClear(runtime.clone()).await,

        "history:getStack" => HistoryGetStack(runtime.clone()).await,

        "label:getUri" => LabelGetURI(runtime.clone(), arguments).await,

        "label:getWorkspace" => LabelGetWorkspace(runtime.clone()).await,

        "label:getBase" => LabelGetBase(arguments).await,

        _ => Err(format!("Unknown navigation command: {}", command)),
    }
}
