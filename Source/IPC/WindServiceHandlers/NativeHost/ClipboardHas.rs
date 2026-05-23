//! Wire method: `nativeHost:hasClipboard`.

use serde_json::{Value, json};

pub async fn Fn(_Arguments:Vec<Value>) -> Result<Value, String> { Ok(json!(false)) }
