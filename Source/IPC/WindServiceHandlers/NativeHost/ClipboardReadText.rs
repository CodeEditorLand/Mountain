//! Wire method: `nativeHost:readClipboardText`.

use serde_json::{Value, json};

pub async fn Fn(_Arguments:Vec<Value>) -> Result<Value, String> {
	match arboard::Clipboard::new() {
		Ok(mut Cb) => Ok(json!(Cb.get_text().unwrap_or_default())),

		Err(_) => Ok(json!("")),
	}
}
