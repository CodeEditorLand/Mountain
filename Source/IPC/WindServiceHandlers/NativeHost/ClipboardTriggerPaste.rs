//! Wire method: `nativeHost:triggerPaste`.
//! Tauri 2.x has no direct paste API - returns false so callers fall
//! through to the OS native paste shortcut.

use serde_json::{Value, json};

pub async fn Fn(_Arguments:Vec<Value>) -> Result<Value, String> { Ok(json!(false)) }
