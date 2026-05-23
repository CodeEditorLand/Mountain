#![allow(unused_variables)]

//! Wire method: `keybinding:getAll`.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let All = RunTime.Environment.ApplicationState.Feature.Keybindings.GetAllKeybindings();

	Ok(json!(All))
}
