//! # CommandProvider
//!
//! Implements `CommandExecutor` for `MountainEnvironment` - the central
//! registry and dispatcher for all commands in Mountain. Commands are
//! identified by string IDs and handled either by native Rust functions
//! or proxied to extension sidecar processes via IPC.
//!
//! ## Execution flow
//!
//! 1. Caller invokes `ExecuteCommand(id, args)`.
//! 2. Provider looks up `id` in `ApplicationState::CommandRegistry`.
//! 3. **Native handler**: calls the Rust function pointer directly with
//!    `AppHandle`, the main `WebviewWindow`, `ApplicationRunTime`, and `args`.
//! 4. **Proxied handler**: sends an RPC request to the owning extension
//!    sidecar via the Vine IPC client.
//! 5. Returns a serialized `serde_json::Value` result or a `CommonError`.
//!
//! ## Special-case no-ops
//!
//! Three categories of unknown commands are silently returned as
//! `Value::Null` rather than an error, to match VS Code's behaviour:
//! - View-action auto-commands (`.focus`, `.resetViewLocation`, `.removeView`)
//! - Workbench-internal commands with no Land backing service
//!   (`getTelemetrySenderObject`, `testing.clearTestResults`)
//! - Bootstrap-phase activation-race commands (`_typescript.*`, etc.)
//!
//! ## Lazy activation
//!
//! If a command is not yet in the registry but a scanned extension declares
//! `onCommand:<id>` as an activation event, `ExecuteCommand` fires
//! `$activateByEvent` to Cocoon, yields 50 ms for the fire-and-forget
//! `registerCommand` notification to arrive, then retries the lookup.
//!
//! ## Backlog
//!
//! - Contribution points from extensions; enablement/disable state
//! - Categories, grouping, aliases, deprecation
//! - History and undo/redo stack; keyboard shortcut resolution
//! - Permission validation; batching for related operations; telemetry
//!
//! VS Code reference: `vs/platform/commands/common/commands.ts`,
//! `vs/workbench/services/commands/common/commandService.ts`.

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{
	Command::CommandExecutor::CommandExecutor,
	Error::CommonError::CommonError,
	IPC::DTO::ProxyTarget::ProxyTarget,
};
use async_trait::async_trait;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime, WebviewWindow};

use super::MountainEnvironment::MountainEnvironment;
use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Vine::Client, dev_log};

/// An enum representing the different ways a command can be handled.
pub enum CommandHandler<R:Runtime + 'static> {
	/// A command handled by a native, asynchronous Rust function.
	Native(
		fn(
			AppHandle<R>,

			WebviewWindow<R>,

			Arc<ApplicationRunTime>,

			Value,
		) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>,
	),

	/// A command implemented in an extension and proxied to a sidecar.
	Proxied { SideCarIdentifier:String, CommandIdentifier:String },
}

impl<R:Runtime> Clone for CommandHandler<R> {
	fn clone(&self) -> Self {
		match self {
			Self::Native(Function) => Self::Native(*Function),

			Self::Proxied { SideCarIdentifier, CommandIdentifier } => {
				Self::Proxied {
					SideCarIdentifier:SideCarIdentifier.clone(),

					CommandIdentifier:CommandIdentifier.clone(),
				}
			},
		}
	}
}

#[async_trait]
impl CommandExecutor for MountainEnvironment {
	/// Executes a registered command by dispatching it to the appropriate
	/// handler.
	async fn ExecuteCommand(&self, CommandIdentifier:String, Argument:Value) -> Result<Value, CommonError> {
		let HandlerInfoOption = self
			.ApplicationState
			.Extension
			.Registry
			.CommandRegistry
			.lock()
			.map_err(super::Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
			.get(&CommandIdentifier)
			.cloned();

		match HandlerInfoOption {
			Some(CommandHandler::Native(Function)) => {
				// Per-execution line. The setContext dominator is already
				// gated in Command/Bootstrap.rs; other native commands
				// (openWalkthrough, etc.) fire rarely enough that the
				// surviving tag volume is low, but `commands-verbose`
				// keeps this opt-in for consistency.
				dev_log!(
					"commands-verbose",
					"[CommandProvider] Executing NATIVE command '{}'.",
					CommandIdentifier
				);

				let RunTime:Arc<ApplicationRunTime> =
					self.ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

				let MainWindow = self.ApplicationHandle.get_webview_window("main").ok_or_else(|| {
					CommonError::UserInterfaceInteraction {
						Reason:"Main window not found for command execution".into(),
					}
				})?;

				Function(self.ApplicationHandle.clone(), MainWindow, RunTime, Argument)
					.await
					.map_err(|Error| CommonError::CommandExecution { CommandIdentifier, Reason:Error })
			},

			Some(CommandHandler::Proxied { SideCarIdentifier, CommandIdentifier: ProxiedCommandIdentifier }) => {
				dev_log!(
					"commands-verbose",
					"[CommandProvider] Executing PROXIED command '{}' on sidecar '{}'.",
					CommandIdentifier,
					SideCarIdentifier
				);

				let RPCParameters = json!([ProxiedCommandIdentifier, Argument]);

				let RPCMethod = format!("{}$ExecuteContributedCommand", ProxyTarget::ExtHostCommands.GetTargetPrefix());

				Client::SendRequest::Fn(&SideCarIdentifier, RPCMethod, RPCParameters, 30000)
					.await
					.map_err(|Error| CommonError::IPCError { Description:Error.to_string() })
			},

			None => {
				// VS Code auto-registers `<viewId>.focus`,
				// `<viewId>.resetViewLocation`, and `<viewId>.removeView`
				// commands when a view is contributed via the view registry.
				// Land's webview.registerView bypasses that registry and
				// emits a Tauri event instead, so the focus commands never
				// get inserted. Extensions (gitlens in particular) call
				// `commands.executeCommand('<their-view-id>.focus')` on
				// user gesture; the Cocoon try/catch swallows the error,
				// but the red `error:` log noise here is misleading. Treat
				// these well-known auto-generated suffixes as silent no-ops.
				if CommandIdentifier.ends_with(".focus")
					|| CommandIdentifier.ends_with(".resetViewLocation")
					|| CommandIdentifier.ends_with(".removeView")
				{
					// Once-per-command-id so the no-op fallback doesn't
					// generate an N-line trail through the dev log every
					// time the user clicks a view-action button. The
					// first occurrence still fires (documents the probe
					// shape); subsequent invocations of the same command
					// are silent.
					crate::IPC::DevLog::DebugOnce::Fn(
						"commands",
						&format!("view-action-noop:{}", CommandIdentifier),
						&format!(
							"[CommandProvider] View-action command '{}' not registered; treating as no-op \
							 (auto-generated by view registry in stock VS Code).",
							CommandIdentifier
						),
					);

					return Ok(Value::Null);
				}

				// Workbench-internal commands that stock VS Code registers on
				// the renderer side via `CommandsRegistry.registerCommand(…)`
				// but that Land doesn't carry because the backing service
				// doesn't exist:
				//
				// - `getTelemetrySenderObject` - `vs/platform/telemetry/**` registers this so
				//   extensions can fetch a `TelemetrySender` via `commands.executeCommand`.
				//   Land has no telemetry backend, so returning null (no sender) matches the
				//   "telemetry disabled" code path every extension already defensively handles.
				// - `testing.clearTestResults` - registered by
				//   `vs/workbench/contrib/testing/browser/testExplorerActions.ts`. No
				//   test-explorer UI in Land today; null is the correct "nothing to clear"
				//   shape.
				//
				// Extensions that look these up defensively try/catch. The
				// only observable effect of the prior error return was the
				// red `error:` log line. Treat as silent no-ops until Land
				// grows the corresponding services.
				if matches!(
					CommandIdentifier.as_str(),
					"getTelemetrySenderObject" | "testing.clearTestResults"
				) {
					// `getTelemetrySenderObject` fires once per extension
					// activation (~30+ times per boot) - same once-per-id
					// dedup as the view-action path so the log line
					// documents the probe but doesn't trail.
					crate::IPC::DevLog::DebugOnce::Fn(
						"commands",
						&format!("workbench-internal-noop:{}", CommandIdentifier),
						&format!(
							"[CommandProvider] Workbench-internal command '{}' not registered; treating as no-op \
							 (Land has no backing service).",
							CommandIdentifier
						),
					);
					return Ok(Value::Null);
				}

				// TOCTOU race: Cocoon's `registerCommand` notification is
				// fire-and-forget async, so Mountain's registry doesn't
				// reflect a just-registered command for several ms. The
				// TypeScript extension's post-activation pipeline invokes
				// `_typescript.configurePlugin` within the same event-loop
				// tick as its own `registerCommand`; the intervening
				// executeCommand finds no handler and we emit an
				// alarming red error: line.
				//
				// These internal-underscore-prefixed commands (the VS Code
				// convention for "not-user-facing, extension-internal")
				// are all bootstrap-phase hooks the extension expects to
				// be safely droppable if the registry hasn't caught up yet.
				// Return Value::Null - the extension's own try/catch
				// takes the expected "not yet available" path. The next
				// user gesture triggers a fresh call that finds the
				// command registered normally.
				if CommandIdentifier.starts_with("_typescript.")
					|| CommandIdentifier.starts_with("_extensionHost.")
					|| CommandIdentifier.starts_with("_workbench.registerWebview")
					|| CommandIdentifier.ends_with(".activationCompleted")
					|| CommandIdentifier.ends_with(".activated")
					|| CommandIdentifier.ends_with(".ready")
				{
					dev_log!(
						"commands",
						"[CommandProvider] Activation-race command '{}' not yet in registry; returning null \
						 (extension will retry post-activation).",
						CommandIdentifier
					);
					return Ok(Value::Null);
				}

				// Lazy activation: stock VS Code fires
				// `$activateByEvent("onCommand:<cmd>")` whenever a
				// command-not-found lookup matches an extension's
				// declared activation events. The extension then
				// registers its command during activation, and the
				// second registry lookup succeeds. Without this flow,
				// any extension that gates on `onCommand:<id>` (e.g.
				// GitLens' primary commands, Roo-Cline's commands, Vim
				// mode toggles) never activates in response to a user
				// gesture - it just silently does nothing.
				if LookupCommandContributingExtension(self, &CommandIdentifier) {
					dev_log!(
						"commands",
						"[CommandProvider] Lazy activation for command '{}' - firing onCommand:{0}",
						CommandIdentifier
					);
					let Event = format!("onCommand:{}", CommandIdentifier);
					let ActivationResult = Client::SendRequest::Fn(
						&"cocoon-main".to_string(),
						"$activateByEvent".to_string(),
						json!({ "activationEvent": Event }),
						30_000,
					)
					.await;
					if let Err(Error) = ActivationResult {
						dev_log!(
							"commands",
							"warn: [CommandProvider] onCommand:{} activation failed: {}",
							CommandIdentifier,
							Error
						);
					}
					// Small yield so Cocoon's fire-and-forget
					// `registerCommand` notification reaches Mountain's
					// registry before the re-poll.
					tokio::time::sleep(std::time::Duration::from_millis(50)).await;
					let PostActivationHandler = self
						.ApplicationState
						.Extension
						.Registry
						.CommandRegistry
						.lock()
						.map_err(super::Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
						.get(&CommandIdentifier)
						.cloned();
					if let Some(Handler) = PostActivationHandler {
						match Handler {
							CommandHandler::Native(Function) => {
								let MainWindow =
									self.ApplicationHandle.get_webview_window("main").ok_or_else(|| {
										CommonError::IPCError {
											Description:"Could not find main window for lazy-activated native command"
												.to_string(),
										}
									})?;
								let RunTime =
									self.ApplicationHandle.try_state::<Arc<ApplicationRunTime>>().ok_or_else(|| {
										CommonError::IPCError {
											Description:"ApplicationRunTime unavailable for lazy-activated native \
											             command"
												.to_string(),
										}
									})?;
								return Function(
									self.ApplicationHandle.clone(),
									MainWindow,
									(*RunTime).clone(),
									Argument,
								)
								.await
								.map_err(|Error| CommonError::CommandExecution { CommandIdentifier, Reason:Error });
							},
							CommandHandler::Proxied { SideCarIdentifier, CommandIdentifier: ProxiedId } => {
								let RPCParameters = json!([ProxiedId, Argument]);
								let RPCMethod = format!(
									"{}$ExecuteContributedCommand",
									ProxyTarget::ExtHostCommands.GetTargetPrefix()
								);
								return Client::SendRequest::Fn(&SideCarIdentifier, RPCMethod, RPCParameters, 30_000)
									.await
									.map_err(|Error| CommonError::IPCError { Description:Error.to_string() });
							},
						}
					}
				}

				dev_log!(
					"commands",
					"error: [CommandProvider] Command '{}' not found in registry.",
					CommandIdentifier
				);

				Err(CommonError::CommandNotFound { Identifier:CommandIdentifier })
			},
		}
	}

	/// Registers a command contributed by a sidecar process.
	async fn RegisterCommand(&self, SideCarIdentifier:String, CommandIdentifier:String) -> Result<(), CommonError> {
		dev_log!(
			"commands",
			"[CommandProvider] Registering PROXY command '{}' from sidecar '{}'",
			CommandIdentifier,
			SideCarIdentifier
		);

		let mut Registry = self
			.ApplicationState
			.Extension
			.Registry
			.CommandRegistry
			.lock()
			.map_err(super::Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

		Registry.insert(
			CommandIdentifier.clone(),
			CommandHandler::Proxied { SideCarIdentifier, CommandIdentifier },
		);

		Ok(())
	}

	/// Unregisters a previously registered command.
	async fn UnregisterCommand(&self, _SideCarIdentifier:String, CommandIdentifier:String) -> Result<(), CommonError> {
		dev_log!("commands", "[CommandProvider] Unregistering command '{}'", CommandIdentifier);

		self.ApplicationState
			.Extension
			.Registry
			.CommandRegistry
			.lock()
			.map_err(super::Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
			.remove(&CommandIdentifier);

		Ok(())
	}

	/// Gets a list of all currently registered command IDs.
	async fn GetAllCommands(&self) -> Result<Vec<String>, CommonError> {
		dev_log!("commands", "[CommandProvider] Getting all command identifiers.");

		let Registry = self
			.ApplicationState
			.Extension
			.Registry
			.CommandRegistry
			.lock()
			.map_err(super::Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

		Ok(Registry.keys().cloned().collect())
	}
}

/// Return `true` when some scanned extension declares
/// `onCommand:<CommandIdentifier>` as one of its activation events. Used
/// by the lazy-activation fallback in `ExecuteCommand` - without this
/// check we'd fire an `$activateByEvent("onCommand:X")` for every
/// unknown command, which would cause Cocoon to log "no extension
/// matching event" for every typo. Scans the cached registry; no IPC.
fn LookupCommandContributingExtension(Environment:&MountainEnvironment, CommandIdentifier:&str) -> bool {
	let Event = format!("onCommand:{}", CommandIdentifier);
	let Guard = match Environment
		.ApplicationState
		.Extension
		.ScannedExtensions
		.ScannedExtensions
		.lock()
	{
		Ok(G) => G,
		Err(_) => return false,
	};
	for Description in Guard.values() {
		if let Some(Events) = &Description.ActivationEvents {
			if Events.iter().any(|E| E == &Event) {
				return true;
			}
		}
	}
	false
}
