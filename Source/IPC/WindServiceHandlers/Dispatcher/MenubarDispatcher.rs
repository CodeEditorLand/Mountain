//! Menubar command dispatcher - fast-path handled, this is fallback.

use serde_json::Value;

/// Dispatches menubar commands.
///
/// Handled commands:
/// - `menubar:updateMenubar` (fast-path already handled, this is fallback)
pub async fn dispatch_menubar(_arguments:Vec<Value>) -> Result<Value, String> {
	use std::sync::atomic::{AtomicU64, Ordering as AO};

	static MENUBAR_CALLS_FAST:AtomicU64 = AtomicU64::new(0);

	let n = MENUBAR_CALLS_FAST.fetch_add(1, AO::Relaxed) + 1;

	if n == 1 || n % 100 == 0 {
		crate::dev_log!("menubar", "menubar:updateMenubar (fast-path call #{})", n);
	}

	Ok(Value::Null)
}
