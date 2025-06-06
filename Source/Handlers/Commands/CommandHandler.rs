
// Defines the CommandHandler enum, which categorizes different types of command
// handlers used within the application, such as native Rust handlers or those
// proxied to a sidecar.

use std::{future::Future, pin::Pin, sync::Arc};

use serde_json::Value;
use tauri::{AppHandle, Runtime, Window}; // Assuming Wry will be specified by R: Runtime

use crate::Runtime::AppRuntime;

/// Enum representing the different ways a command can be handled.
pub enum CommandHandler<R:Runtime + 'static> {
	/// A command handled by a native Rust function.
	/// The function signature is designed to be flexible, receiving the Tauri
	/// AppHandle, the specific Window the command might be related to (if
	/// any), the application's Runtime, and a generic Value for arguments.
	Native(
		fn(
			AppHandle<R>,
			Window<R>,
			Arc<AppRuntime>,
			Value, // Argument for the command
		) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>,
	),
	/// A command that is proxied to a sidecar process (e.g., Cocoon extension
	/// host).
	Proxied {
		#[serde(alias = "sidecarId")]
		SidecarIdentifier:String, // The identifier of the target sidecar
		#[serde(alias = "commandId")]
		CommandIdentifier:String, // The command ID as known by the sidecar
	},
}

/// Implements Clone for CommandHandler to allow instances to be duplicated.
/// This is necessary if CommandHandler instances are stored in shared
/// structures or passed around in a way that requires ownership transfer or
/// duplication.
impl<R:Runtime + 'static> Clone for CommandHandler<R> {
	fn clone(&self) -> Self {
		match self {
			CommandHandler::Native(FunctionPointer) => CommandHandler::Native(*FunctionPointer),
			CommandHandler::Proxied { SidecarIdentifier, CommandIdentifier } => {
				CommandHandler::Proxied {
					SidecarIdentifier:SidecarIdentifier.clone(),
					CommandIdentifier:CommandIdentifier.clone(),
				}
			},
		}
	}
}
