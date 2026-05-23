//! Wire method: `update:downloadUpdate`.
//! No-op - Land has no update server.

use serde_json::Value;

pub async fn Fn() -> Result<Value, String> {
	crate::dev_log!("update", "update:downloadUpdate");

	Ok(Value::Null)
}
