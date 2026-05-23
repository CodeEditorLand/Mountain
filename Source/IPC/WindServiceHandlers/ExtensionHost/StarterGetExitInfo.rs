#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `extensionHostStarter:getExitInfo`.
//! Returns stub exit-info shape - Cocoon runs while Mountain is alive.

use serde_json::{Value, json};

pub async fn Fn(_Arguments:Vec<Value>) -> Result<Value, String> {
	crate::dev_log!("exthost", "extensionHostStarter:getExitInfo");

	Ok(json!({ "code": null, "signal": null }))
}
