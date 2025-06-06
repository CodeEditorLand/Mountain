
// Defines the RPC handler for terminal-related operations requested by the
// sidecar. This includes creating, showing, hiding, sending text to, and
// disposing of terminals.

use std::sync::Arc;

use log::{debug, info, trace};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, State, Wry};

// Assuming AppRuntimeTrait is not strictly needed if all logic is in Handlers::Terminal
// use Common::Runtime::AppRuntimeTrait;
// use Common::Errors::CommonError;
use crate::Runtime::AppRuntime; // Kept for consistency if other methods need it
use crate::{
	Handlers::{self, ErrorUtils, Terminal as TerminalHandlerLogic},
	Rpc::Argument::Terminal::{
		CreateTerminalArgument,
		IdArgument as TerminalIdentifierArgument,
		SendTextArgument as SendTextToTerminalArgument,
		ShowArgument as ShowTerminalArgument,
	},
};

#[derive(Clone)]
pub struct MainThreadTerminalServiceHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	// Runtime might not be strictly needed if all logic is in Handlers::Terminal
	pub Runtime:Arc<AppRuntime>,
}

impl MainThreadTerminalServiceHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry>, Runtime:Arc<AppRuntime>) -> Self {
		Self { ApplicationHandle, Runtime }
	}

	/// Creates a new terminal instance.
	pub async fn CreateTerminal(&self, Argument:CreateTerminalArgument) -> Result<Value, String> {
		info!("[Rpc TerminalServiceHandler] CreateTerminal (DTO): Name='{:?}'", Argument.Name);
		// The original `handle_create_terminal` in `Handler/terminal.rs` expects a
		// Value. We need to serialize our DTO back to Value for it.
		let ParametersValueForHandler = serde_json::to_value(Argument).map_err(|SerializationError| {
			ErrorUtils::RpcInternalErrorString(format!(
				"Failed to re-serialize CreateTerminalArgument DTO: {}",
				SerializationError
			))
		})?;
		TerminalHandlerLogic::HandleCreateTerminal(self.ApplicationHandle.clone(), ParametersValueForHandler).await
	}

	/// Shows a specific terminal.
	pub async fn Show(&self, Argument:ShowTerminalArgument) -> Result<Value, String> {
		info!(
			"[Rpc TerminalServiceHandler] Show (DTO): Identifier={}, PreserveFocus={:?}",
			Argument.Id, Argument.PreserveFocus
		);
		// Original handler expects a Value which is an array: [id, preserveFocus?]
		let ParametersValueForHandler = json!([Argument.Id, Argument.PreserveFocus]);
		TerminalHandlerLogic::HandleShowTerminal(self.ApplicationHandle.clone(), ParametersValueForHandler).await
	}

	/// Hides a specific terminal.
	pub async fn Hide(&self, Argument:TerminalIdentifierArgument) -> Result<Value, String> {
		info!("[Rpc TerminalServiceHandler] Hide (DTO): Identifier={}", Argument.Id);
		// Original handler expects a Value which is an array: [id]
		let ParametersValueForHandler = json!([Argument.Id]);
		TerminalHandlerLogic::HandleHideTerminal(self.ApplicationHandle.clone(), ParametersValueForHandler).await
	}

	/// Sends text input to a specific terminal.
	pub async fn SendText(&self, Argument:SendTextToTerminalArgument) -> Result<Value, String> {
		info!(
			"[Rpc TerminalServiceHandler] SendText (DTO): Identifier={}, TextLength={}, AddNewLine={:?}",
			Argument.Id,
			Argument.Text.len(),
			Argument.AddNewLine
		);
		// Original handler expects a Value which is an array: [id, text, addNewLine?]
		let ParametersValueForHandler = json!([Argument.Id, Argument.Text, Argument.AddNewLine]);
		TerminalHandlerLogic::HandleSendTextToTerminal(self.ApplicationHandle.clone(), ParametersValueForHandler).await
	}

	/// Disposes of (closes and cleans up) a specific terminal.
	pub async fn Dispose(&self, Argument:TerminalIdentifierArgument) -> Result<Value, String> {
		info!("[Rpc TerminalServiceHandler] Dispose (DTO): Identifier={}", Argument.Id);
		// Original handler expects a Value which is an array: [id]
		let ParametersValueForHandler = json!([Argument.Id]);
		TerminalHandlerLogic::HandleDisposeTerminal(self.ApplicationHandle.clone(), ParametersValueForHandler).await
	}
}
