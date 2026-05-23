//! Wire method: `nativeHost:writeClipboardText`.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_string;

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Text = arg_string(&Arguments, 0);

	if let Ok(mut Cb) = arboard::Clipboard::new() {
		let _ = Cb.set_text(Text);
	}

	Ok(Value::Null)
}
