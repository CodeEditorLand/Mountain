//! Wire method: `nativeHost:readClipboardFindText`.
//! macOS has a separate find pasteboard; reuse the general clipboard for
//! parity with VS Code on Linux/Windows.

use serde_json::{Value, json};

pub async fn Fn(_Arguments:Vec<Value>) -> Result<Value, String> {

	match arboard::Clipboard::new() {
		Ok(mut Cb) => Ok(json!(Cb.get_text().unwrap_or_default())),

		Err(_) => Ok(json!("")),
	}
}
