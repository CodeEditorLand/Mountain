#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `nativeHost:writeClipboardBuffer`.
//! Binary clipboard not yet implemented - no-op.

use serde_json::Value;

pub async fn Fn(_Arguments:Vec<Value>) -> Result<Value, String> { Ok(Value::Null) }
