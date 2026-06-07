//! TextFile dispatcher.

use crate::Model::{
    TextfileRead::Fn as TextfileRead,
    TextfileWrite::Fn as TextfileWrite,
    TextfileSave::Fn as TextfileSave,
};
use serde_json::Value;

/// Dispatches text file commands.
pub async fn dispatch_text_file(
    runtime: &crate::RunTime::ApplicationRunTime::ApplicationRunTime,
    command: &str,
    arguments: Vec<Value>,
) -> Result<Value, String> {
    match command {
        "textFile:read" => TextfileRead(runtime.clone(), arguments).await,
        "textFile:write" => TextfileWrite(runtime.clone(), arguments).await,
        "textFile:save" => TextfileSave(runtime.clone(), arguments).await,
        _ => Err(format!("Unknown text file command: {}", command)),
    }
}
