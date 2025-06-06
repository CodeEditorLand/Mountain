// File: Rpc/MainThreadMessageHandler.rs
// Defines the RPC handler for requests from the sidecar to show messages
// (information, warnings, errors) to the user, typically via native dialogs or
// UI notifications.

use std::sync::Arc;

use Common::UiEffect::{self, MessageSeverity as CommonMessageSeverity}; // Assuming this path and enum
use Common::{Errors::CommonError, Runtime::AppRuntimeTrait};
use log::{debug, info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Wry};

use crate::{Handlers::ErrorUtils, Runtime::AppRuntime};

#[derive(Clone)]
pub struct MainThreadMessageHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	pub Runtime:Arc<AppRuntime>,
}

impl MainThreadMessageHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry>, Runtime:Arc<AppRuntime>) -> Self {
		Self { ApplicationHandle, Runtime }
	}

	/// Shows a message to the user.
	/// The `ArgumentsValue` is expected to be an array where:
	/// - `args[0]` is the severity level (u64, mapped to MessageSeverity).
	/// - `args[1]` is the message string.
	/// - `args[2]` (optional) is a DTO for message options (e.g., buttons,
	///   modality).
	pub async fn ShowMessage(&self, ArgumentsValue:Value) -> Result<Value, String> {
		let ParametersArray = ArgumentsValue
			.as_array()
			.ok_or_else(|| ErrorUtils::RpcParamErrorString("ShowMessage", "ArgumentsValue", "array", None))?;

		let SeverityNumber = ParametersArray.get(0).and_then(Value::as_u64).ok_or_else(|| {
			ErrorUtils::RpcParamErrorString("ShowMessage", "Severity (args[0])", "u64 number", Some(0))
		})?;

		let MessageString = ParametersArray
			.get(1)
			.and_then(Value::as_str)
			.ok_or_else(|| ErrorUtils::RpcParamErrorString("ShowMessage", "Message (args[1])", "string", Some(1)))?
			.to_string();

		let OptionsValueOption = ParametersArray.get(2).cloned();

		info!(
			"[Rpc MainThreadMessageHandler] ShowMessage: SeverityNumber={}, MessageLength={}, OptionsPresent={}",
			SeverityNumber,
			MessageString.len(),
			OptionsValueOption.is_some()
		);

		// Map VS Code severity numbers (if that's what's coming) to
		// CommonMessageSeverity VS Code severity: 1=Info, 2=Warning, 3=Error.
		// (Assuming these are VS Code severity levels from the original `track.rs`)
		let EffectSeverity = match SeverityNumber {
			1 => CommonMessageSeverity::Info,
			2 => CommonMessageSeverity::Warning,
			3 => CommonMessageSeverity::Error,
			SeverityUnknown => {
				warn!(
					"[Rpc MainThreadMessageHandler] Unknown severity number {} from ShowMessage. Defaulting to Info.",
					SeverityUnknown
				);
				CommonMessageSeverity::Info
			},
		};

		let ShowMessageEffect = UiEffect::ShowMessage(
			EffectSeverity,
			MessageString,
			OptionsValueOption.unwrap_or(Value::Null), // Pass options as Value to the effect
		);

		self.Runtime.Run(ShowMessageEffect).await.map_err(|CommonErrorValue| {
			ErrorUtils::MapCommonErrorToRpcString(CommonErrorValue, "ShowMessage DTO effect execution")
		})
	}
}
