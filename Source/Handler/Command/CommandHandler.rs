// @module CommandHandler
// @description Defines the `CommandHandler` enum, which is used by the command
// registry to categorize and store the implementation details for each
// command.

use std::{future::Future, pin::Pin, sync::Arc};

use serde_json::Value;
use tauri::{AppHandle, Runtime, Window};
RunTime
use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

// An enum representing the different ways a command can be handled by the
// system. This allows the command dispatcher to decide whether to execute a
// local Rust function or to proxy the request to an external sidecar process.
pub enum CommandHandler<R:Runtime + 'static> {
	// A command handled by a native, asynchronous Rust function. The function
	// pointer has a standardized signature to receive all necessary context.
	Native(
		fn(
			AppHandle<R>,
			Window<R>,
			Arc<ApplicationRunTime>,
			Value,
		) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>,
	),
	// A command that is implemented in an extension and must be proxied to a
	// sidecar process (like Cocoon) for execution.
	Proxied { SidecarIdentifier:String, CommandIdentifier:String },
}

impl<R:Runtime + 'static> Clone for CommandHandler<R> {
	fn clone(&self) -> Self {
		match self {
			CommandHandler::Native(function_pointer) => CommandHandler::Native(*function_pointer),
			CommandHandler::Proxied { SidecarIdentifier, CommandIdentifier } => {
				CommandHandler::Proxied {
					SidecarIdentifier:SidecarIdentifier.clone(),
					CommandIdentifier:CommandIdentifier.clone(),
				}
			},
		}
	}
}
