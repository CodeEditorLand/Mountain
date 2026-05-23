//! Wire method: `nativeHost:writeClipboardText`.

use serde_json::Value;

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Text = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	if let Ok(mut Cb) = arboard::Clipboard::new() {
		let _ = Cb.set_text(Text);
	}

	Ok(Value::Null)
}
