
// Defines the RPC handler for log messages received from the sidecar.
// These logs are typically from the extension host environment (e.g., Cocoon).

use std::sync::Arc;

use log::{LevelFilter, debug, error, info, trace, warn}; // Added LevelFilter
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Wry};

use crate::Handlers::ErrorUtils;
use crate::Runtime::AppRuntime; // Likely not needed if only logging

#[derive(Clone)]
pub struct MainThreadLogHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	// Runtime might not be needed if this handler only logs.
	// pub Runtime: Arc<AppRuntime>,
}

impl MainThreadLogHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry> /* , Runtime: Arc<AppRuntime> */) -> Self {
		Self { ApplicationHandle /* , Runtime */ }
	}

	/// Processes a log message from the sidecar.
	/// The `ArgumentValue` is expected to be an array where:
	/// - `args[0]` is the log level (u64, mapped to LevelFilter).
	/// - `args[1]` is an array of message parts to be joined, or a single
	///   message string.
	pub async fn Log(&self, ArgumentValue:Value) -> Result<Value, String> {
		let ArgumentArray = ArgumentValue
			.as_array()
			.ok_or_else(|| ErrorUtils::RpcParamErrorString("Log", "ArgumentValue", "array", None))?;

		let LogLevelNumber = ArgumentArray.get(0).and_then(Value::as_u64).unwrap_or(2); // Default to Info (VS Code level 2)

		let MessagePartsValue = ArgumentArray.get(1).cloned().unwrap_or_else(|| json!([]));

		let MessageString = if let Some(PartsArray) = MessagePartsValue.as_array() {
			PartsArray
				.iter()
				.map(|ValuePart| ValuePart.as_str().unwrap_or_else(|| ValuePart.to_string()))
				.collect::<Vec<_>>()
				.join(" ")
		} else {
			MessagePartsValue
				.as_str()
				.unwrap_or_else(|| MessagePartsValue.to_string())
				.to_string()
		};

		// Map VS Code log level numbers to `log::LevelFilter` or `log::Level`
		// VS Code levels: 0=Trace, 1=Debug, 2=Info, 3=Warn, 4=Error, 5=Critical/Off
		match LogLevelNumber {
			0 => trace!("[Cocoon ExtHost Log RPC] {}", MessageString), // Trace
			1 => debug!("[Cocoon ExtHost Log RPC] {}", MessageString), // Debug
			2 => info!("[Cocoon ExtHost Log RPC] {}", MessageString),  // Info
			3 => warn!("[Cocoon ExtHost Log RPC] {}", MessageString),  // Warn
			4 | 5 => error!("[Cocoon ExtHost Log RPC] {}", MessageString), // Error / Critical
			_ => info!("[Cocoon ExtHost Log RPC] (Unknown Level {}) {}", LogLevelNumber, MessageString),
		}

		Ok(Value::Null)
	}
}
