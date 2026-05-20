#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire methods: `extensionHostStarter:*`.
//! Bridges VS Code's `IExtensionHostStarter` channel to Mountain/Cocoon.
//! `createExtensionHost` allocates a stub id; `start` returns Cocoon's real
//! PID so debuggers attach to the correct Node.js process.

use serde_json::{Value, json};

pub async fn ExtensionHostStarterCreate(_Arguments:Vec<Value>) -> Result<Value, String> {
	crate::dev_log!("exthost", "extensionHostStarter:createExtensionHost");
	Ok(json!({ "id": "1" }))
}

pub async fn ExtensionHostStarterStart(_Arguments:Vec<Value>) -> Result<Value, String> {
	// The renderer uses this PID to correlate extension-host debug adapters
	// with the actual Node.js process. That process is Cocoon, not Mountain -
	// returning `std::process::id()` here would point the debugger at
	// Mountain's Rust binary. Fall back to Mountain's PID only if Cocoon
	// hasn't spawned yet (should not happen for a real extension-host start).
	let Pid = crate::ProcessManagement::CocoonManagement::GetCocoonPid().unwrap_or_else(std::process::id);
	crate::dev_log!("exthost", "extensionHostStarter:start pid={}", Pid);
	Ok(json!({ "pid": Pid }))
}

pub async fn ExtensionHostStarterKill(_Arguments:Vec<Value>) -> Result<Value, String> {
	crate::dev_log!("exthost", "extensionHostStarter:kill");
	Ok(Value::Null)
}

pub async fn ExtensionHostStarterGetExitInfo(_Arguments:Vec<Value>) -> Result<Value, String> {
	crate::dev_log!("exthost", "extensionHostStarter:getExitInfo");
	Ok(json!({ "code": null, "signal": null }))
}
