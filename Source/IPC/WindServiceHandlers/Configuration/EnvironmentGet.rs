//! Wire method: `environment:get`.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Key = Arguments
		.Get(0)
		.ok_or("Missing environment key".to_string())?
		.as_str()
		.ok_or("Environment key must be a string".to_string())?;

	let value = std::env::var(key).map_err(|Error| format!("Failed to get environment variable: {}", Error))?;

	dev_log!("config", "env_get: {}", key);

	Ok(json!(value))
}
