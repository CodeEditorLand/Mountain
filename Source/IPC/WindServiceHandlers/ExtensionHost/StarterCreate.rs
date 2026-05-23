
//! Wire method: `extensionHostStarter:createExtensionHost`.
//! Allocates a stub extension-host ID for VS Code's starter protocol.

use serde_json::{Value, json};

pub async fn Fn(_Arguments:Vec<Value>) -> Result<Value, String> {
	crate::dev_log!("exthost", "extensionHostStarter:createExtensionHost");

	Ok(json!({ "id": "1" }))
}
