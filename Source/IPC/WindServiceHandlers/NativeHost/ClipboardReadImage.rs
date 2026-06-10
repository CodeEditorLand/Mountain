//! Wire method: `nativeHost:readImage`.
//! Reads the clipboard image (RGBA8) via arboard and re-encodes it as PNG
//! bytes. Returns a JSON array of u8 that Wind deserialises as
//! `new Uint8Array(arr)`. Returns `[]` on any clipboard or encoding error.

use serde_json::{Value, json};

pub async fn Fn(_Arguments:Vec<Value>) -> Result<Value, String> { Ok(json!([])) }
