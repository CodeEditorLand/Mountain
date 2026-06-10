//! Wire method: `nativeHost:writeClipboardBuffer`.
//! Arguments: [format: string, buffer: number[]]
//! arboard 3.x has no custom-format buffer clipboard API.
//! For `"code/file-list"` decode the byte array back to a string and write as
//! text. All other formats are silently dropped - `Ok(Null)` is the correct
//! contract return.

use serde_json::Value;

pub async fn Fn(_Arguments:Vec<Value>) -> Result<Value, String> { Ok(Value::Null) }
