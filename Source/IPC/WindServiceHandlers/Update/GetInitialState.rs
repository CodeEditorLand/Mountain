//! Wire method: `update:_getInitialState`.
//! Returns `{ type: "idle" }` so the workbench renders "up to date".

use serde_json::{Value, json};

pub async fn Fn() -> Result<Value, String> {
	crate::dev_log!("update", "update:_getInitialState");

	Ok(json!({ "type": "idle", "updateType": 0 }))
}
