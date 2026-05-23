//! Wire method: `update:applyUpdate`.
//! No-op - Land has no update server.

use serde_json::Value;

pub async fn Fn() -> Result<Value, String> {
	crate::dev_log!("update", "update:applyUpdate");

	Ok(Value::Null)
}
