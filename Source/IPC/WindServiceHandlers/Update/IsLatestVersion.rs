#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `update:isLatestVersion`.
//! Always returns `true` - Land has no update server.

use serde_json::{Value, json};

pub async fn Fn() -> Result<Value, String> {
	crate::dev_log!("update", "update:isLatestVersion");

	Ok(json!(true))
}
