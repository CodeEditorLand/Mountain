//! Logger command dispatcher - handles all logger:* and legacy log:* commands.

use serde_json::Value;

use crate::Utilities::JsonValueHelpers::{Fn as v_str, arg_string};

/// Dispatches logger and legacy log commands.
///
/// Handled commands:
/// - `logger:log`, `logger:info`, `logger:debug`, `logger:trace`
/// - `logger:warn`, `logger:error`, `logger:critical`
/// - `logger:flush`, `logger:setLevel`, `logger:getLevel`
/// - `logger:createLogger`, `logger:registerLogger`
/// - `logger:deregisterLogger`, `logger:getRegisteredLoggers`
/// - `logger:setVisibility`
/// - `log:registerLogger`, `log:createLogger` (legacy)
/// - `storage:onDidChangeItems`, `storage:logStorage` (stubs)
/// - `commands:registerCommand`, `commands:unregisterCommand`
/// - `commands:onDidRegisterCommand`, `commands:onDidExecuteCommand`
/// - `configuration:onDidChange` (stub)
/// - `storage:optimize`, `storage:isUsed`, `storage:close` (stubs)
/// - `workspaces:onDidChangeWorkspaceFolders`
/// - `workspaces:onDidChangeWorkspaceName`
pub async fn dispatch_logger(command:&str, arguments:&[Value]) -> Result<Value, String> {
	// Extract log level from command
	let level = command.trim_start_matches("logger:").trim_start_matches("log:");

	// Build message from arguments
	let msg = if arguments.len() >= 2 {
		let tail:Vec<String> = arguments
			.iter()
			.skip(1)
			.filter_map(|v| v.as_str().map(str::to_owned).or_else(|| serde_json::to_string(v).ok()))
			.collect();

		tail.join(" ")
	} else {
		arguments
			.first()
			.and_then(|v| v.as_str().map(str::to_owned))
			.unwrap_or_default()
	};

	if !msg.is_empty() {
		match level {
			"error" | "critical" => crate::dev_log!("vscode-log", "[ERROR] {}", msg),

			"warn" => crate::dev_log!("vscode-log", "[WARN] {}", msg),

			_ => crate::dev_log!("vscode-log", "{}", msg),
		}
	}

	Ok(Value::Null)
}

/// Dispatches fast-path no-op commands that just return Null.
///
/// Handled commands (all return Ok(Value::Null)):
/// - `logger:flush`, `logger:setLevel`, `logger:getLevel`
/// - `logger:createLogger`, `logger:registerLogger`
/// - `logger:deregisterLogger`, `logger:getRegisteredLoggers`
/// - `logger:setVisibility`
/// - `log:registerLogger`, `log:createLogger`
/// - `storage:onDidChangeItems`, `storage:logStorage`
/// - `commands:registerCommand`, `commands:unregisterCommand`
/// - `commands:onDidRegisterCommand`, `commands:onDidExecuteCommand`
/// - `configuration:onDidChange`
/// - `storage:optimize`, `storage:isUsed`, `storage:close`
/// - `workspaces:onDidChangeWorkspaceFolders`
/// - `workspaces:onDidChangeWorkspaceName`
pub async fn dispatch_noop(_command:&str, _arguments:&[Value]) -> Result<Value, String> { Ok(Value::Null) }
