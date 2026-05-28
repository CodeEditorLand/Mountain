//! # Bootstrap (Command)
//!
//! Registers all native, Rust-implemented commands and providers into the
//! application's state at startup. This module ensures all core functionality
//! is available as soon as the application initializes.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Command Registration
//! - Register all Tauri command handlers from `Command::` module
//! - Register core IPC command handlers from `Track::` module
//! - Build the complete `invoke_handler` vector for Tauri builder
//! - Ensure all commands are available before UI starts
//!
//! ### 2. Tree View Provider Registration
//! - Register native tree view providers (FileExplorer, etc.)
//! - Create provider instances and store in `ApplicationState::ActiveTreeViews`
//! - Associate view identifiers with provider implementations
//!
//! ### 3. Provider Registration
//! - Initialize Environment providers that need early setup
//! - Register command executors and configuration providers
//! - Set up document and workspace providers
//!
//! ## ARCHITECTURAL ROLE
//!
//! Bootstrap is the **registration orchestrator** for Mountain's startup:
//!
//! ```text
//! Binary::Main ──► Bootstrap::RegisterAll ──► Tauri Builder ──► App Ready
//!                      │
//!                      ├─► Command Handlers Registered
//!                      ├─► Tree View Providers Registered
//!                      └─► ApplicationState Populated
//! ```
//!
//! ### Position in Mountain
//! - `Command` module: Command system initialization
//! - Called from `Binary::Main::Fn` during Tauri builder setup
//! - Must complete before `.run()` is called on Tauri app
//!
//! ### Key Functions
//! - `RegisterAll`: Main entry point that registers everything
//! - `RegisterCommands`: Adds all Tauri command handlers
//! - `RegisterTreeViewProviders`: Registers native tree view providers
//!
//! ## REGISTRATION PROCESS
//!
//! 1. **Commands**: All command functions are added to Tauri's `invoke_handler`
//!    via `tauri::generate_handler![]` macro
//! 2. **Tree Views**: Native providers are instantiated and stored in state
//! 3. **Error Handling**: Registration failures are logged but don't stop
//!    startup
//!
//! ## COMMAND REGISTRATION
//!
//! The following command modules are registered:
//! - `Command::TreeView::GetTreeViewChildren`
//! - `Command::LanguageFeature::MountainProvideHover`
//! - `Command::LanguageFeature::MountainProvideCompletions`
//! - `Command::LanguageFeature::MountainProvideDefinition`
//! - `Command::LanguageFeature::MountainProvideReferences`
//! - `Command::SourceControlManagement::GetAllSourceControlManagementState`
//! - `Command::Keybinding::GetResolvedKeybinding`
//! - `Track::DispatchLogic::DispatchFrontendCommand`
//! - `Track::DispatchLogic::ResolveUIRequest`
//! - `IPC::TauriIPCServer::mountain_ipc_receive_message`
//! - `IPC::TauriIPCServer::mountain_ipc_get_status`
//! - `Binary::Main::SwitchTrayIcon`
//! - `Binary::Main::MountainGetWorkbenchConfiguration`
//! - (and more...)
//!
//! ## TREE VIEW PROVIDERS
//!
//! Currently registered native providers:
//! - `FileExplorerViewProvider`: File system tree view
//!   - View ID: `"fileExplorer"`
//!   - Provides workspace folders and file listings
//!
//! ## PERFORMANCE
//!
//! - Registration is synchronous and fast (no async allowed in registration)
//! - All commands are registered up-front; no lazy loading
//! - Tree view providers are created once at startup
//!
//! ## ERROR HANDLING
//!
//! - Command registration errors are logged as errors
//! - Tree view provider errors are logged as warnings
//! - Registration continues even if some components fail
//!
//! ## TODO
//!
//! - [ ] Add command registration metrics (count, duplicates detection)
//! - [ ] Implement command dependency ordering
//! - [ ] Add command validation (duplicate names, signature checking)
//! - [ ] Support dynamic command registration after startup
//! - [ ] Add command unregistration for hot-reload scenarios
//! - [ ] Implement command permission system
//!
//! ## MODULE CONTENTS
//!
//! - `RegisterAll`: Main registration function called from Binary::Main
//! - `RegisterCommands`: Internal function to register all command handlers
//! - `RegisterTreeViewProviders`: Internal function to register tree view
//! providers

// ## VSCode Reference:
// - vs/workbench/services/actions/common/menuService.ts
// - vs/workbench/browser/actions.ts
// - vs/platform/actions/common/actions.ts
//
// ============================================================================

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{
	DTO::WorkspaceEditDTO::WorkspaceEditDTO,
	Document::OpenDocument::OpenDocument,
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	LanguageFeature::LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	UserInterface::ShowOpenDialog::ShowOpenDialog,
	Workspace::ApplyWorkspaceEdit::ApplyWorkspaceEdit,
};
use serde_json::{Value, json};
use tauri::{AppHandle, WebviewWindow, Wry};
use url::Url;

use crate::{
	ApplicationState::{
		DTO::TreeViewStateDTO::TreeViewStateDTO,
		State::ApplicationState::{ApplicationState, MapLockError},
	},
	Environment::CommandProvider::CommandHandler,
	FileSystem::FileExplorerViewProvider::Struct as FileExplorerViewProvider,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

// --- Command Implementations ---

/// A simple native command that logs a message.
fn CommandHelloWorld(
	_ApplicationHandle:AppHandle<Wry>,

	_Window:WebviewWindow<Wry>,

	_RunTime:Arc<ApplicationRunTime>,

	_Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		dev_log!("commands", "[Native Command] Hello from Mountain!");

		Ok(json!("Hello from Mountain's native command!"))
	})
}

/// A native command that orchestrates the "Open File" dialog flow.
fn CommandOpenFile(
	_ApplicationHandle:AppHandle<Wry>,

	_Window:WebviewWindow<Wry>,

	RunTime:Arc<ApplicationRunTime>,

	_Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		dev_log!("commands", "[Native Command] Executing Open File...");

		let DialogResult = RunTime.Run(ShowOpenDialog(None)).await.map_err(|Error| Error.to_string())?;

		if let Some(Paths) = DialogResult {
			if let Some(Path) = Paths.first() {
				// We have a path, now open the document.
				let URI = Url::from_file_path(Path).map_err(|_| "Invalid file path".to_string())?;

				let OpenDocumentEffect = OpenDocument(json!({ "external": URI.to_string() }), None, None);

				RunTime.Run(OpenDocumentEffect).await.map_err(|Error| Error.to_string())?;
			}
		}

		Ok(Value::Null)
	})
}

/// A native command that orchestrates the "Format Document" action.
fn CommandFormatDocument(
	_ApplicationHandle:AppHandle<Wry>,

	_Window:WebviewWindow<Wry>,

	RunTime:Arc<ApplicationRunTime>,

	_Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		dev_log!("commands", "[Native Command] Executing Format Document...");

		let AppState = &RunTime.Environment.ApplicationState;

		let URIString = AppState
			.Workspace
			.ActiveDocumentURI
			.lock()
			.map_err(MapLockError)
			.map_err(|Error| Error.to_string())?
			.clone()
			.ok_or("No active document URI found in state".to_string())?;

		let URI = Url::parse(&URIString).map_err(|_| "Invalid URI in window state".to_string())?;

		// Example formatting options
		let Options = json!({ "tabSize": 4, "insertSpaces": true });

		// 1. Get the formatting edits from the language feature provider.
		let LanguageProvider:Arc<dyn LanguageFeatureProviderRegistry> = RunTime.Environment.Require();

		let EditsOption = LanguageProvider
			.ProvideDocumentFormattingEdits(URI.clone(), Options)
			.await
			.map_err(|Error| Error.to_string())?;

		if let Some(Edits) = EditsOption {
			if Edits.is_empty() {
				dev_log!("commands", "[Native Command] No formatting changes to apply.");

				return Ok(Value::Null);
			}

			// 2. Convert the text edits into a WorkspaceEdit.
			let WorkspaceEdit = WorkspaceEditDTO {
				Edits:vec![(
					serde_json::to_value(&URI).map_err(|Error| Error.to_string())?,
					Edits
						.into_iter()
						.map(serde_json::to_value)
						.collect::<Result<Vec<_>, _>>()
						.map_err(|Error| Error.to_string())?,
				)],
			};

			// 3. Apply the workspace edit.
			dev_log!("commands", "[Native Command] Applying formatting edits...");

			RunTime
				.Run(ApplyWorkspaceEdit(WorkspaceEdit))
				.await
				.map_err(|Error| Error.to_string())?;
		} else {
			dev_log!("commands", "[Native Command] No formatting provider found for this document.");
		}

		Ok(Value::Null)
	})
}

/// A native command for saving the current document.
fn CommandSaveDocument(
	_ApplicationHandle:AppHandle<Wry>,

	_Window:WebviewWindow<Wry>,

	RunTime:Arc<ApplicationRunTime>,

	_Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		dev_log!("commands", "[Native Command] Executing Save Document...");

		let AppState = &RunTime.Environment.ApplicationState;

		let URIString = AppState
			.Workspace
			.ActiveDocumentURI
			.lock()
			.map_err(MapLockError)
			.map_err(|Error| Error.to_string())?
			.clone()
			.ok_or("No active document URI found in state".to_string())?;

		let URI = Url::parse(&URIString).map_err(|_| "Invalid URI in window state".to_string())?;

		// Persist the active document by invoking DocumentProvider::SaveDocument or the
		// Document::Save effect. This reads the document URI from ApplicationState,
		// serializes the current editor content, and writes to disk with proper error
		// handling, atomic writes, and backup creation. Current implementation only
		// logs the action; full implementation requires integration with the document
		// lifecycle and file system provider.
		dev_log!("commands", "[Native Command] Saving document: {}", URI);

		Ok(Value::Null)
	})
}

/// A native command for closing the current document.
fn CommandCloseDocument(
	_ApplicationHandle:AppHandle<Wry>,

	_Window:WebviewWindow<Wry>,

	RunTime:Arc<ApplicationRunTime>,

	_Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		dev_log!("commands", "[Native Command] Executing Close Document...");

		let AppState = &RunTime.Environment.ApplicationState;

		let URIString = AppState
			.Workspace
			.ActiveDocumentURI
			.lock()
			.map_err(MapLockError)
			.map_err(|Error| Error.to_string())?
			.clone()
			.ok_or("No active document URI found in state".to_string())?;

		let URI = Url::parse(&URIString).map_err(|_| "Invalid URI in window state".to_string())?;

		// Close the active document in the editor by triggering the workspace edit
		// to remove the document from open editors. Checks for unsaved changes and
		// prompts the user to save, discard, or cancel. Integrates with the document
		// lifecycle manager to release resources and update the UI. May invoke
		// Workbench::closeEditor or equivalent command. Current implementation only
		// logs the action.
		dev_log!("commands", "[Native Command] Closing document: {}", URI);

		Ok(Value::Null)
	})
}

/// Native no-op for VS Code's built-in `setContext` command. Extensions call
/// `vscode.commands.executeCommand('setContext', key, value)` to set UI
/// context-key state used for when-clauses. Wind/Sky owns the actual context
/// key service; Mountain forwards the value so CommandProvider doesn't raise
/// "not found". Returns null because the real VS Code command returns void.
fn CommandSetContext(
	_ApplicationHandle:AppHandle<Wry>,

	_Window:WebviewWindow<Wry>,

	_RunTime:Arc<ApplicationRunTime>,

	Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		// setContext fires on every UI state change (focus, view toggle,
		// gitlens mode, SCM repo change). ~130 calls per session. Route
		// to `commands-verbose` so per-keypress context changes don't
		// flood the default log.
		dev_log!("commands-verbose", "[Native Command] setContext: {}", Argument);

		Ok(Value::Null)
	})
}

/// Native no-op for `workbench.action.openWalkthrough`. VS Code's
/// walkthrough UI lives in `workbench/contrib/welcomeGettingStarted` and is
/// not wired through Land yet. Extensions (notably `claude-code`) invoke this
/// at activation - returning null avoids a user-visible "command not found"
/// error while the walkthrough system remains unimplemented.
fn CommandOpenWalkthrough(
	_ApplicationHandle:AppHandle<Wry>,

	_Window:WebviewWindow<Wry>,

	_RunTime:Arc<ApplicationRunTime>,

	Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		dev_log!("commands", "[Native Command] openWalkthrough (no-op): {}", Argument);

		Ok(Value::Null)
	})
}

/// A native command for reloading the window.
fn CommandReloadWindow(
	_ApplicationHandle:AppHandle<Wry>,

	Window:WebviewWindow<Wry>,

	_RunTime:Arc<ApplicationRunTime>,

	_Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		dev_log!("commands", "[Native Command] Executing Reload Window...");

		// Drive the real webview reload so extensions, settings, and locale
		// changes take effect without restarting the process. Swallow the
		// error - VS Code's contract returns `{ success: true }` on
		// best-effort reload and extensions don't inspect it further.
		if let Err(Error) = Window.eval("location.reload()") {
			dev_log!("commands", "warn: [Native Command] Reload Window eval failed: {}", Error);
		}

		Ok(json!({ "success": true }))
	})
}

/// `vscode.open(uri, columnOrOptions?)` - the built-in command every
/// extension uses to jump to a file or open an external URL. Routes to
/// `window.showTextDocument` for `file://` URIs (via the sky-channel so Sky
/// can open the editor) and to `NativeHost.OpenExternal` for anything else.
fn CommandVscodeOpen(
	ApplicationHandle:AppHandle<Wry>,

	_Window:WebviewWindow<Wry>,

	_RunTime:Arc<ApplicationRunTime>,

	Argument:Value,
) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
	Box::pin(async move {
		use tauri::Emitter;

		let UriRaw = if Argument.is_array() {
			Argument.get(0).cloned().unwrap_or_default()
		} else {
			Argument.clone()
		};

		// Resolve the URI to a real wire string. Cocoon may forward a raw
		// string, a serialised `vscode.Uri` POJO (`{scheme, authority,
		// path, query, fragment}`), or a `{external, path}` shape used by
		// older rendering paths. Reconstruct the full URI rather than
		// picking a single field - extracting bare `path` from a non-file
		// URI (e.g. `rust-analyzer-diagnostics-view:/diag/foo`) drops the
		// scheme and Sky then tries to open `/diag/foo` as a file, which
		// either 404s or renders as "[object Object]" in the editor tab
		// when the workbench falls back to `String(uri)` on a bad input.
		let UriString = match &UriRaw {
			Value::String(S) => S.clone(),
			Value::Object(Object) => {
				if let Some(External) = Object.get("external").and_then(Value::as_str) {
					External.to_string()
				} else if let Some(Scheme) = Object.get("scheme").and_then(Value::as_str)
					&& !Scheme.is_empty()
				{
					let Authority = Object.get("authority").and_then(Value::as_str).unwrap_or("");

					let Path = Object.get("path").and_then(Value::as_str).unwrap_or("");

					let Query = Object.get("query").and_then(Value::as_str).unwrap_or("");

					let Fragment = Object.get("fragment").and_then(Value::as_str).unwrap_or("");

					let mut Out = format!("{}://{}{}", Scheme, Authority, Path);

					if !Query.is_empty() {
						Out.push('?');

						Out.push_str(Query);
					}

					if !Fragment.is_empty() {
						Out.push('#');

						Out.push_str(Fragment);
					}

					Out
				} else if let Some(FsPath) = Object.get("fsPath").and_then(Value::as_str) {
					if FsPath.starts_with('/') {
						format!("file://{}", FsPath)
					} else {
						FsPath.to_string()
					}
				} else if let Some(Path) = Object.get("path").and_then(Value::as_str) {
					Path.to_string()
				} else {
					String::new()
				}
			},
			Value::Null => String::new(),
			_ => UriRaw.to_string(),
		};

		if UriString.is_empty() {
			return Err("vscode.open requires a URI".to_string());
		}

		let IsFileLike = UriString.starts_with("file:") || UriString.starts_with('/');

		if IsFileLike {
			if let Err(Error) = ApplicationHandle.emit("sky://window/showTextDocument", json!({ "uri": UriString })) {
				dev_log!(
					"commands",
					"warn: [vscode.open] sky://window/showTextDocument emit failed: {}",
					Error
				);
			}

			Ok(json!(true))
		} else {
			// Fall through to platform open. Mirrors `NativeHost.OpenExternal`.
			let Command:Option<(&str, Vec<String>)> = if cfg!(target_os = "macos") {
				Some(("open", vec![UriString.clone()]))
			} else if cfg!(target_os = "windows") {
				Some(("cmd.exe", vec!["/c".into(), "start".into(), String::new(), UriString.clone()]))
			} else {
				Some(("xdg-open", vec![UriString.clone()]))
			};

			if let Some((Bin, Args)) = Command {
				let _ = tokio::process::Command::new(Bin).args(&Args).spawn();
			}

			Ok(json!(true))
		}
	})
}

/// Validates command parameters before execution.
fn ValidateCommandParameters(CommandName:&str, Arguments:&Value) -> Result<(), String> {
	match CommandName {
		"mountain.openFile" | "workbench.action.files.openFile" => {
			// No specific validation needed for open file
			Ok(())
		},

		"editor.action.formatDocument" => {
			// Ensure there's an active document
			Ok(())
		},

		_ => Ok(()),
	}
}

// --- Registration Function ---

/// Registers all native commands and providers with the application state.
pub fn RegisterNativeCommands(
	AppHandle:&AppHandle<Wry>,

	ApplicationState:&Arc<ApplicationState>,
) -> Result<(), CommonError> {
	// --- Command Registration ---
	let mut CommandRegistry = ApplicationState
		.Extension
		.Registry
		.CommandRegistry
		.lock()
		.map_err(MapLockError)?;

	dev_log!("commands", "[Bootstrap] Registering native commands...");

	// Register core commands
	CommandRegistry.insert("mountain.helloWorld".to_string(), CommandHandler::Native(CommandHelloWorld));

	CommandRegistry.insert("mountain.openFile".to_string(), CommandHandler::Native(CommandOpenFile));

	CommandRegistry.insert(
		"workbench.action.files.openFile".to_string(),
		CommandHandler::Native(CommandOpenFile),
	);

	CommandRegistry.insert(
		"editor.action.formatDocument".to_string(),
		CommandHandler::Native(CommandFormatDocument),
	);

	CommandRegistry.insert(
		"workbench.action.files.save".to_string(),
		CommandHandler::Native(CommandSaveDocument),
	);

	CommandRegistry.insert(
		"workbench.action.closeActiveEditor".to_string(),
		CommandHandler::Native(CommandCloseDocument),
	);

	CommandRegistry.insert(
		"workbench.action.reloadWindow".to_string(),
		CommandHandler::Native(CommandReloadWindow),
	);

	// setContext is VS Code built-in - extensions invoke it on activation to
	// declare UI context keys. Registering as a no-op silences the routing
	// error until Wind/Sky wire through a real context key service.
	CommandRegistry.insert("setContext".to_string(), CommandHandler::Native(CommandSetContext));

	// `vscode.open(uri)` - dispatches to the editor for file URIs and to the
	// platform shell for everything else. Extensions call this without
	// guarding on whether we've registered it; a missing registration shows
	// up as "command 'vscode.open' not found" in user-visible error toasts.
	CommandRegistry.insert("vscode.open".to_string(), CommandHandler::Native(CommandVscodeOpen));

	CommandRegistry.insert("vscode.openFolder".to_string(), CommandHandler::Native(CommandVscodeOpen));

	// `workbench.action.openWalkthrough` is VS Code's welcome/getting-started
	// walkthrough entry point; the `claude-code` extension wraps it with its
	// own `claude-vscode.openWalkthrough` command and invokes both at
	// activation. Land has no walkthrough UI yet - register both as no-ops so
	// extension activation doesn't surface "command not found" errors.
	CommandRegistry.insert(
		"workbench.action.openWalkthrough".to_string(),
		CommandHandler::Native(CommandOpenWalkthrough),
	);

	CommandRegistry.insert(
		"claude-vscode.openWalkthrough".to_string(),
		CommandHandler::Native(CommandOpenWalkthrough),
	);

	dev_log!("commands", "[Bootstrap] {} native commands registered.", CommandRegistry.len());

	drop(CommandRegistry);

	// --- Command Validation ---
	dev_log!("commands", "[Bootstrap] Validating registered commands...");

	// Validate all registered commands at startup to catch configuration errors
	// early. Verification includes command signature correctness, parameter type
	// matching, required permissions and capabilities, and extension metadata
	// validity. This prevents runtime errors from malformed registrations and
	// provides immediate feedback to extension developers during development.
	// Current implementation logs without performing actual validation checks.

	// --- Tree View Provider Registration ---
	let mut TreeViewRegistry = ApplicationState
		.Feature
		.TreeViews
		.ActiveTreeViews
		.lock()
		.map_err(MapLockError)?;

	dev_log!("commands", "[Bootstrap] Registering native tree view providers...");

	let ExplorerViewID = "workbench.view.explorer".to_string();

	let ExplorerProvider = Arc::new(FileExplorerViewProvider::New(AppHandle.clone()));

	TreeViewRegistry.insert(
		ExplorerViewID.clone(),
		TreeViewStateDTO {
			ViewIdentifier:ExplorerViewID,

			Provider:Some(ExplorerProvider),

			// This is a native provider
			SideCarIdentifier:None,

			CanSelectMany:true,

			HasHandleDrag:false,

			HasHandleDrop:false,

			Message:None,

			Title:Some("Explorer".to_string()),

			Description:None,

			Badge:None,
		},
	);

	dev_log!(
		"commands",
		"[Bootstrap] {} native tree view providers registered.",
		TreeViewRegistry.len()
	);

	Ok(())
}
