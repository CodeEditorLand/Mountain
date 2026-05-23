#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `extensionHostStarter:waitForExit`.
//! Resolves when the extension host exits. Returns stub exit-info so callers
//! do not hang - Cocoon runs indefinitely while Mountain is alive.

use serde_json::{Value, json};

pub async fn Fn(_Arguments:Vec<Value>) -> Result<Value, String> {
	crate::dev_log!("exthost", "extensionHostStarter:waitForExit");

	Ok(json!({ "code": null, "signal": null }))
}
