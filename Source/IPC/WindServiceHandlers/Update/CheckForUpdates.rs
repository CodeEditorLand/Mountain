#![allow(unused_variables, dead_code, unused_imports)]

//! Wire method: `update:checkForUpdates`.
//! No-op - Land has no update server.

use serde_json::Value;

pub async fn Fn() -> Result<Value, String> {
	crate::dev_log!("update", "update:checkForUpdates");

	Ok(Value::Null)
}
