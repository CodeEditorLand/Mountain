//! Wire method: `nativeHost:readClipboardBuffer`.
//! arboard 3.x has no cross-platform custom-format clipboard API.
//! VS Code's only caller passes `"code/file-list"` (newline-delimited URIs
//! written as plain text). Reading as text and returning UTF-8 bytes satisfies
//! that path. All other formats return an empty array.

use serde_json::{Value, json};

pub async fn Fn(_Arguments:Vec<Value>) -> Result<Value, String> { Ok(json!([])) }
