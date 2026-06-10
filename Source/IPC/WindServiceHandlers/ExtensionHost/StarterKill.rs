//! Wire method: `extensionHostStarter:kill`.
//! Acknowledged no-op - Cocoon lifecycle is managed by Mountain directly.

use serde_json::Value;

pub async fn Fn(_Arguments:Vec<Value>) -> Result<Value, String> {
	crate::dev_log!("exthost", "extensionHostStarter:kill");

	Ok(Value::Null)
}
