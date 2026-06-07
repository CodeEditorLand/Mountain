//! Keybinding command dispatcher.

use crate::UI::{
    KeybindingAdd::Fn as KeybindingAdd,
    KeybindingGetAll::Fn as KeybindingGetAll,
    KeybindingLookup::Fn as KeybindingLookup,
    KeybindingRemove::Fn as KeybindingRemove,
};

use serde_json::Value;

/// Dispatches keybinding commands.
///
/// Handled commands:
/// - `keybinding:add`
/// - `keybinding:remove`
/// - `keybinding:lookup`
/// - `keybinding:getAll`
pub async fn dispatch_keybinding(
    runtime: &crate::RunTime::ApplicationRunTime::ApplicationRunTime,

    command: &str,

    arguments: Vec<Value>,
) -> Result<Value, String> {

    match command {
        "keybinding:add" => KeybindingAdd(runtime.clone(), arguments).await,

        "keybinding:remove" => KeybindingRemove(runtime.clone(), arguments).await,

        "keybinding:lookup" => KeybindingLookup(runtime.clone(), arguments).await,

        "keybinding:getAll" => KeybindingGetAll(runtime.clone()).await,

        _ => Err(format!("Unknown keybinding command: {}", command)),
    }
}
