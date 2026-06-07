//! URL dispatcher.

use serde_json::Value;

/// Dispatches URL commands.
pub async fn dispatch_url(
    _command: &str,
) -> Result<Value, String> {
    // url:registerExternalUriOpener - stub
    Ok(Value::Null)
}
