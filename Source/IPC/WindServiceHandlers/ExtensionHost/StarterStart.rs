#![allow(unused_variables, dead_code, unused_imports)]

//! Wire method: `extensionHostStarter:start`.
//! Returns Cocoon's real PID so debuggers attach to the correct Node.js
//! process. Falls back to Mountain's PID only if Cocoon has not spawned yet.

use serde_json::{Value, json};

pub async fn Fn(_Arguments:Vec<Value>) -> Result<Value, String> {
	let Pid = crate::ProcessManagement::CocoonManagement::GetCocoonPid().unwrap_or_else(std::process::id);

	crate::dev_log!("exthost", "extensionHostStarter:start pid={}", Pid);

	Ok(json!({ "pid": Pid }))
}
