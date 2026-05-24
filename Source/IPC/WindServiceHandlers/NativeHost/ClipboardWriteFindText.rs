//! Wire method: `nativeHost:writeClipboardFindText`.

use serde_json::Value;

use crate::IPC::WindServiceHandlers::Utilities::JsonValueHelpers::ArgString;

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Text = ArgString(&Arguments, 0);

	if let Ok(mut Cb) = arboard::Clipboard::new() {
		let _ = Cb.set_text(Text);
	}

	Ok(Value::Null)
}
