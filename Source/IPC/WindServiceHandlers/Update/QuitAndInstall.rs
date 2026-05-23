#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `update:quitAndInstall`.
//! No-op - Land has no update server.

use serde_json::Value;

pub async fn Fn() -> Result<Value, String> {
	crate::dev_log!("update", "update:quitAndInstall");

	Ok(Value::Null)
}
