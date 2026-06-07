//! Output command dispatcher.

use crate::Output::{
    OutputAppend::Fn as OutputAppend,
    OutputAppendLine::Fn as OutputAppendLine,
    OutputClear::Fn as OutputClear,
    OutputCreate::Fn as OutputCreate,
    OutputShow::Fn as OutputShow,
};

use serde_json::Value;

/// Dispatches output commands.
///
/// Handled commands:
/// - `output:create`
/// - `output:append`
/// - `output:appendLine`
/// - `output:clear`
/// - `output:show`
pub async fn dispatch_output(
    app_handle: &tauri::AppHandle,

    command: &str,

    arguments: Vec<Value>,
) -> Result<Value, String> {

    match command {
        "output:create" => OutputCreate(app_handle.clone(), arguments).await,

        "output:append" => OutputAppend(app_handle.clone(), arguments).await,

        "output:appendLine" => OutputAppendLine(app_handle.clone(), arguments).await,

        "output:clear" => OutputClear(app_handle.clone(), arguments).await,

        "output:show" => OutputShow(app_handle.clone(), arguments).await,

        _ => Err(format!("Unknown output command: {}", command)),
    }
}
