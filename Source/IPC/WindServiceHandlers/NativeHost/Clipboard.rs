#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire methods: clipboard operations via `nativeHost:*Clipboard*`.
//! Backed by `arboard` for cross-platform text clipboard access.
//! Binary clipboard (`readClipboardBuffer`, `writeClipboardBuffer`) returns
//! empty/null - binary clipboard is rarely used by VS Code core.

use serde_json::{Value, json};

pub async fn NativeReadClipboardText(_Arguments:Vec<Value>) -> Result<Value, String> {
	match arboard::Clipboard::new() {
		Ok(mut Cb) => Ok(json!(Cb.get_text().unwrap_or_default())),

		Err(_) => Ok(json!("")),
	}
}

pub async fn NativeWriteClipboardText(Arguments:Vec<Value>) -> Result<Value, String> {
	let Text = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	if let Ok(mut Cb) = arboard::Clipboard::new() {
		let _ = Cb.set_text(Text);
	}

	Ok(Value::Null)
}

/// macOS has a separate find pasteboard; reuse the general clipboard for
/// parity with VS Code on Linux/Windows.
pub async fn NativeReadClipboardFindText(_Arguments:Vec<Value>) -> Result<Value, String> {
	match arboard::Clipboard::new() {
		Ok(mut Cb) => Ok(json!(Cb.get_text().unwrap_or_default())),

		Err(_) => Ok(json!("")),
	}
}

pub async fn NativeWriteClipboardFindText(Arguments:Vec<Value>) -> Result<Value, String> {
	let Text = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	if let Ok(mut Cb) = arboard::Clipboard::new() {
		let _ = Cb.set_text(Text);
	}

	Ok(Value::Null)
}

pub async fn NativeReadClipboardBuffer(_Arguments:Vec<Value>) -> Result<Value, String> { Ok(json!([])) }

pub async fn NativeWriteClipboardBuffer(_Arguments:Vec<Value>) -> Result<Value, String> { Ok(Value::Null) }

pub async fn NativeHasClipboard(_Arguments:Vec<Value>) -> Result<Value, String> { Ok(json!(false)) }

/// Trigger a paste operation. On Tauri 2.x there is no direct `paste` API;
/// return false so callers fall through to the OS's native paste shortcut.
pub async fn NativeTriggerPaste(_Arguments:Vec<Value>) -> Result<Value, String> { Ok(json!(false)) }

/// Read an image from the clipboard. Binary clipboard is not yet
/// implemented - return an empty array so callers get a safe fallback.
pub async fn NativeReadImage(_Arguments:Vec<Value>) -> Result<Value, String> { Ok(json!([])) }
