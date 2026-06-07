//! Progress command dispatcher.

use crate::UI::{
    ProgressBegin::Fn as ProgressBegin,
    ProgressEnd::Fn as ProgressEnd,
    ProgressReport::Fn as ProgressReport,
};

use serde_json::Value;

/// Dispatches progress commands.
///
/// Handled commands:
/// - `progress:begin`
/// - `progress:report`
/// - `progress:end`
pub async fn dispatch_progress(
    app_handle: &tauri::AppHandle,

    command: &str,

    arguments: Vec<Value>,
) -> Result<Value, String> {

    match command {
        "progress:begin" => ProgressBegin(app_handle.clone(), arguments).await,

        "progress:report" => ProgressReport(app_handle.clone(), arguments).await,

        "progress:end" => ProgressEnd(app_handle.clone(), arguments).await,

        _ => Err(format!("Unknown progress command: {}", command)),
    }
}
