#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wind Service Handlers - dispatcher and sub-module aggregator.
//! Domain files handle the individual handler implementations.

pub mod Commands;
pub mod Configuration;
pub mod Extension;
pub mod Extensions;
pub mod FileSystem;
pub mod Git;
pub mod Model;
pub mod NativeDialog;
pub mod NativeHost;
pub mod Navigation;
pub mod Output;
pub mod Search;
pub mod Storage;
pub mod Terminal;
pub mod UI;
pub mod Utilities;

// Local `use X::*;` (NOT `pub use`): brings the domain handler names into
// this file's scope so the dispatch match arms below can call
// `handle_foo(...)` unqualified. Local `use` is scoped to this file only;
// external callers must spell the full path
// (`WindServiceHandlers::Utilities::foo`).
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use Commands::*;
use Configuration::*;
use Extensions::*;
use FileSystem::{
	Managed::{
		FileCopy::*,
		FileDelete::*,
		FileExists::*,
		FileMkdir::*,
		FileMove::*,
		FileRead::*,
		FileReadBinary::*,
		FileReaddir::*,
		FileStat::*,
		FileWrite::*,
		FileWriteBinary::*,
	},
	Native::{
		FileCloneNative::*,
		FileDeleteNative::*,
		FileExistsNative::*,
		FileMkdirNative::*,
		FileReadNative::*,
		FileReaddirNative::*,
		FileRealpath::*,
		FileRenameNative::*,
		FileStatNative::*,
		FileWriteNative::*,
	},
};
use Model::*;
use NativeHost::{
	FindFreePort::*,
	GetColorScheme::*,
	IsFullscreen::*,
	IsMaximized::*,
	OSProperties::*,
	OSStatistics::*,
	OpenExternal::*,
	PickFolder::*,
	ShowItemInFolder::*,
	ShowOpenDialog::*,
};
use Navigation::*;
use Output::*;
use Search::*;
use Storage::*;
use Terminal::*;
use UI::{
	Decoration::*,
	Keybinding::*,
	Lifecycle::*,
	Notification::*,
	Progress::*,
	QuickInput::*,
	Theme::*,
	WorkingCopy::*,
	Workspace::*,
};
use Utilities::{
	ApplicationRoot::*,
	ChannelPriority::*,
	JsonValueHelpers::*,
	MetadataEncoding::*,
	PathExtraction::*,
	RecentlyOpened::*,
	UserdataDir::*,
};
use Echo::Task::Priority::Priority as EchoPriority;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager};
// Type aliases for Configuration DTOs to simplify usage
use CommonLibrary::Configuration::DTO::{
	ConfigurationOverridesDTO as ConfigurationOverridesDTOModule,
	ConfigurationTarget as ConfigurationTargetModule,
};

use crate::dev_log;
type ConfigurationOverridesDTO = ConfigurationOverridesDTOModule::ConfigurationOverridesDTO;
type ConfigurationTarget = ConfigurationTargetModule::ConfigurationTarget;

use CommonLibrary::{
	Command::CommandExecutor::CommandExecutor,
	Configuration::ConfigurationProvider::ConfigurationProvider,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	ExtensionManagement::ExtensionManagementService::ExtensionManagementService,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
	IPC::SkyEvent::SkyEvent,
	Storage::StorageProvider::StorageProvider,
};

use crate::{
	ApplicationState::{
		ApplicationState,
		DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
		State::WorkspaceState::WorkspaceDelta::UpdateWorkspaceFoldersAndBroadcast,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

/// Internal dispatcher for the single front-end Tauri command
/// `MountainIPCInvoke` (registered in `Binary/Main/Entry.rs::invoke_handler!`,
/// implemented in `Binary/IPC/InvokeCommand.rs`). The outer Tauri command
/// receives `(method: String, params: Value)`, unwraps `params` into a
/// `Vec<Value>`, then delegates here.
///
/// This function is **not** a Tauri command itself - removing the previously
/// present `#[tauri::command]` attribute avoids the false impression that
/// `mountain_ipc_invoke` is reachable from the webview under its snake-case
/// name. All front-end callers (Wind, Sky, Output) must `invoke(
/// "MountainIPCInvoke", { method, params })` through `InvokeCommand::
/// MountainIPCInvoke`; this inner function is pure Rust-side plumbing.
///
/// The local parameter names (`command` / `args`) are preserved for diff
/// minimality; the frontend-facing contract (`method` / `params`) lives
/// entirely in `InvokeCommand.rs`.
pub async fn mountain_ipc_invoke(app_handle:AppHandle, command:String, args:Vec<Value>) -> Result<Value, String> {
	let OTLPStart = crate::IPC::DevLog::NowNano();
	// Silence the per-call invoke log for high-frequency methods that are
	// not useful in forensic review. The workbench emits thousands of
	// `logger:log` invocations per boot (every `console.*` call inside VS
	// Code code becomes an IPC round-trip); keeping those lines only
	// expands log volume without adding signal. The actual dispatch below
	// still runs - this just skips the `[DEV:IPC] invoke:` line.
	//
	// Filesystem methods were driving 13k+ IPC lines per session
	// (FileSystem.ReadFile alone fires thousands of times during svelte
	// / language-server activation). Same for `storage:getItems`,
	// `configuration:lookup`, `themes:getColorTheme` which workbench
	// services poll on every re-render. Add them to the quiet list -
	// the Cocoon gRPC layer + TauriInvoke still log errors, and the IPC
	// `done:` line below also skips these so there's symmetric silence.
	let IsHighFrequencyCommand = matches!(
		command.as_str(),
		"logger:log"
			| "logger:registerLogger"
			| "logger:createLogger"
			| "log:registerLogger"
			| "log:createLogger"
			| "file:stat"
			| "file:readFile"
			| "file:readdir"
			| "file:writeFile"
			| "file:delete"
			| "file:rename"
			| "file:realpath"
			| "file:read"
			| "file:write"
			| "storage:getItems"
			| "storage:updateItems"
			| "configuration:lookup"
			| "configuration:inspect"
			| "themes:getColorTheme"
			| "output:append"
			| "progress:report"
	);
	if !IsHighFrequencyCommand {
		dev_log!("ipc", "invoke: {} args_count={}", command, args.len());
	}

	// Ensure userdata directories exist on first IPC call
	ensure_userdata_dirs();

	// Get the application runtime - deref the Tauri State into an owned Arc
	// so we can hand it to an Echo scheduler task below (State<T> isn't
	// Send across task boundaries).
	let runtime:Arc<ApplicationRunTime> = app_handle.state::<Arc<ApplicationRunTime>>().inner().clone();

	// =========================================================================
	// Route dispatch - every arm has a dev_log! with a granular tag.
	// Tags match the route prefix: vfs, config, storage, extensions,
	// terminal, output, textfile, notification, progress, quickinput,
	// workspaces, themes, search, decorations, workingcopy, keybinding,
	// lifecycle, label, model, history, commands, nativehost, window,
	// exthost, encryption, menubar, update, url, grpc.
	// Activate: Trace=all   or   Trace=vfs,ipc,config
	//
	// Atom O1 + O3: every invoke flows through `SubmitToEcho` below so the
	// Echo work-stealing scheduler picks a lane based on `Channel::Priority()`.
	// The dispatch match still runs inline - Echo's real value is queuing
	// decisions under load, not moving a single future across threads. This
	// keeps the 4900-line match legible while guaranteeing every inbound
	// IPC hits the scheduler's priority machinery on its way out.
	// =========================================================================

	// Tag the pending IPC with its priority lane and submit the entire
	// dispatch future to Echo. Results flow back through a oneshot channel
	// so the Tauri caller still awaits a plain `Result<Value, String>`.
	let CommandPriority = ResolveChannelPriority(&command);

	let Scheduler = runtime.Scheduler.clone();

	let (ResultSender, ResultReceiver) = tokio::sync::oneshot::channel::<Result<Value, String>>();

	let DispatchAppHandle = app_handle.clone();

	let DispatchRuntime = runtime.clone();

	let DispatchCommand = command.clone();

	let DispatchArgs = args;

	Scheduler.Submit(
		async move {
			let app_handle = DispatchAppHandle;
			let runtime = DispatchRuntime;
			let command = DispatchCommand;
			let args = DispatchArgs;

			let MatchResult = match command.as_str() {
				// Configuration commands. VS Code's stock
				// `ConfigurationService` channel calls `getValue` /
				// `updateValue`; Mountain's native Effect-TS layer calls
				// `get` / `update`. Alias both to the same handler so
				// traffic from either rail lands in the same place.
				"configuration:get" | "configuration:getValue" => {
					dev_log!("config", "{}", command);
					handle_configuration_get(runtime.clone(), args).await
				},
				"configuration:update" | "configuration:updateValue" => {
					dev_log!("config", "{}", command);
					handle_configuration_update(runtime.clone(), args).await
				},
				// `ConfigurationService` listens for `onDidChange` from
				// the channel on the binary IPC rail. Mountain broadcasts
				// config changes via a Tauri event directly; ack the
				// channel-listen with Null so the ChannelClient doesn't
				// leak a pending promise.
				"configuration:onDidChange" => Ok(Value::Null),

				// Logger commands - fire-and-forget from Wind, just acknowledge
				"logger:log"
				| "logger:warn"
				| "logger:error"
				| "logger:info"
				| "logger:debug"
				| "logger:trace"
				| "logger:critical"
				| "logger:flush"
				| "logger:setLevel"
				| "logger:getLevel"
				| "logger:createLogger"
				| "logger:registerLogger"
				| "logger:deregisterLogger"
				| "logger:getRegisteredLoggers"
				| "logger:setVisibility" => Ok(Value::Null),

				// File system commands - use native handlers with URI support.
				//
				// The primary names (`file:read`, `file:write`, `file:move`)
				// match Mountain's original dispatch table and are what
				// Wind's Effect-TS layer calls. VS Code's
				// `DiskFileSystemProviderClient` (reached through the
				// binary IPC bridge in Output/IPCRendererShim) uses the
				// stock channel-client method names `readFile`,
				// `writeFile`, `rename`; aliasing them here keeps both
				// rails pointing at the same handler without duplicating
				// logic or introducing a per-caller translation table.
				"file:read" | "file:readFile" => handle_file_read_native(args).await,
				"file:write" | "file:writeFile" => handle_file_write_native(args).await,
				"file:stat" => handle_file_stat_native(args).await,
				"file:exists" => handle_file_exists_native(args).await,
				"file:delete" => handle_file_delete_native(args).await,
				"file:copy" => handle_file_clone_native(args).await,
				"file:move" | "file:rename" => handle_file_rename_native(args).await,
				"file:mkdir" => handle_file_mkdir_native(args).await,
				"file:readdir" => handle_file_readdir_native(args).await,
				"file:readBinary" => handle_file_read_binary(runtime.clone(), args).await,
				"file:writeBinary" => handle_file_write_binary(runtime.clone(), args).await,
				// File watcher channel methods - `DiskFileSystemProvider`
				// opens `watch` / `unwatch` channel calls to receive
				// `onDidChangeFile` events. Until the Mountain-side
				// filewatcher bridge is wired through the binary IPC we
				// ack with Null so the workbench proceeds without a
				// hanging promise.
				"file:watch" | "file:unwatch" => {
					dev_log!("fs-route", "{} (stub-ack)", command);
					Ok(Value::Null)
				},

				// Storage commands. VS Code's
				// `ApplicationStorageDatabaseClient` channel methods are
				// `getItems` / `updateItems` / `optimize` / `close` /
				// `isUsed`; the shorter `storage:get` / `storage:set` are
				// Mountain-native conveniences. All route through the
				// same ApplicationState storage backing.
				"storage:get" => handle_storage_get(runtime.clone(), args).await,
				"storage:set" => handle_storage_set(runtime.clone(), args).await,
				"storage:getItems" => {
					// Workbench services poll this on every theme / scope
					// change; suppress the bare banner and rely on the IPC
					// `invoke:`/`done:` summary for volume + latency.
					dev_log!("storage-verbose", "storage:getItems");
					handle_storage_get_items(runtime.clone(), args).await
				},
				"storage:updateItems" => {
					dev_log!("storage-verbose", "storage:updateItems");
					handle_storage_update_items(runtime.clone(), args).await
				},
				"storage:optimize" => {
					dev_log!("storage", "storage:optimize");
					Ok(Value::Null)
				},
				"storage:isUsed" => {
					dev_log!("storage", "storage:isUsed");
					Ok(Value::Null)
				},
				"storage:close" => {
					dev_log!("storage", "storage:close");
					Ok(Value::Null)
				},
				// Stock VS Code exposes `onDidChangeItems` as a channel
				// event. Ack the listen-request; real change delivery is
				// via Tauri event elsewhere.
				"storage:onDidChangeItems" | "storage:logStorage" => {
					dev_log!("storage-verbose", "{} (stub-ack)", command);
					Ok(Value::Null)
				},

				// Environment commands
				"environment:get" => {
					dev_log!("config", "environment:get");
					handle_environment_get(runtime.clone(), args).await
				},

				// Native host commands
				"native:showItemInFolder" => handle_show_item_in_folder(runtime.clone(), args).await,
				"native:openExternal" => handle_open_external(runtime.clone(), args).await,

				// Workbench commands
				"workbench:getConfiguration" => handle_workbench_configuration(runtime.clone(), args).await,

				// Diagnostic: webview → Mountain dev-log bridge.
				// First arg is a tag ("boot", "extService", …), second is the
				// message, rest are optional structured fields we stringify.
				// Atom H1c: added so workbench.js can surface diagnostic state
				// into the same Mountain.dev.log that carries Rust-side events.
				"diagnostic:log" => {
					let Tag = args.first().and_then(|V| V.as_str()).unwrap_or("webview").to_string();
					let Message = args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();
					let Extras = if args.len() > 2 {
						let Tail:Vec<String> = args
							.iter()
							.skip(2)
							.map(|V| {
								let S = serde_json::to_string(V).unwrap_or_default();
								// Char-aware truncation - JSON-encoded values may
								// embed multi-byte UTF-8 (extension names, repo
								// paths with non-ASCII, debug payloads). Slicing
								// at a fixed byte offset can land mid-codepoint
								// and panic the tokio worker.
								if S.len() > 240 {
									let CutAt = S
										.char_indices()
										.map(|(Index, _)| Index)
										.take_while(|Index| *Index <= 240)
										.last()
										.unwrap_or(0);
									format!("{}…", &S[..CutAt])
								} else {
									S
								}
							})
							.collect();
						format!(" {}", Tail.join(" "))
					} else {
						String::new()
					};
					dev_log!("diagnostic", "[{}] {}{}", Tag, Message, Extras);
					Ok(Value::Null)
				},

				// Command registry commands. Stock VS Code
				// `MainThreadCommands` / `CommandService` channel methods
				// are `executeCommand` and `getCommands`; Mountain's
				// Effect-TS rail uses `execute` / `getAll`. Alias both.
				"commands:execute" | "commands:executeCommand" => handle_commands_execute(runtime.clone(), args).await,
				"commands:getAll" | "commands:getCommands" => {
					dev_log!("commands", "{}", command);
					handle_commands_get_all(runtime.clone()).await
				},
				// Register/unregister from a side-car channel perspective
				// is a no-op: Cocoon sends `$registerCommand` via gRPC
				// (handled elsewhere). Ack Null so the workbench side
				// doesn't hang on a promise.
				"commands:registerCommand"
				| "commands:unregisterCommand"
				| "commands:onDidRegisterCommand"
				| "commands:onDidExecuteCommand" => Ok(Value::Null),

				// Extension host commands
				"extensions:getAll" => {
					dev_log!("extensions", "extensions:getAll");
					handle_extensions_get_all(runtime.clone()).await
				},
				"extensions:get" => {
					dev_log!("extensions", "extensions:get");
					handle_extensions_get(runtime.clone(), args).await
				},
				"extensions:isActive" => {
					dev_log!("extensions", "extensions:isActive");
					handle_extensions_is_active(runtime.clone(), args).await
				},

				// VS Code's Extensions sidebar →
				// `ExtensionManagementChannelClient.getInstalled` goes through
				// `sharedProcessService.getChannel('extensions')`. Sky's
				// astro.config.ts Step 7b swaps the native SharedProcessService
				// for a TauriMainProcessService-backed shim, so the call lands
				// here as `extensions:getInstalled`. The expected return is
				// `ILocalExtension[]` - a wrapper around each scanned manifest
				// with `identifier.id`, `manifest`, `location`, `isBuiltin`, etc.
				// `handle_extensions_get_installed` builds that envelope;
				// `handle_extensions_get_all` returns the raw manifest for
				// callers (Cocoon, Wind Effect services) that want the flat
				// shape. Do NOT alias these two - the payload shapes differ.
				"extensions:getInstalled" | "extensions:scanSystemExtensions" => {
					// Atom H1a: args[0]=type, args[1]=profileLocation URI,
					// args[2]=productVersion, args[3]=??? (VS Code canonical is
					// 3; shim appears to add a 4th). Dump to find out what it
					// contains on post-nav page reloads where the sidebar
					// renders 0 entries despite Mountain returning 94.
					let ArgsSummary = args
						.iter()
						.enumerate()
						.map(|(Idx, V)| {
							let Preview = serde_json::to_string(V).unwrap_or_default();
							// Char-aware truncation - same UTF-8 hazard as
							// the diagnostic-tag formatter above.
							let Trimmed = if Preview.len() > 180 {
								let CutAt = Preview
									.char_indices()
									.map(|(Index, _)| Index)
									.take_while(|Index| *Index <= 180)
									.last()
									.unwrap_or(0);
								format!("{}…", &Preview[..CutAt])
							} else {
								Preview
							};
							format!("[{}]={}", Idx, Trimmed)
						})
						.collect::<Vec<_>>()
						.join(" ");
					dev_log!("extensions", "{} args={}", command, ArgsSummary);
					// `scanSystemExtensions` is conceptually
					// `getInstalled(type=ExtensionType.System)`, so override
					// `args[0]` to `0` before forwarding. Without the override
					// a plain alias would inherit whatever the caller passed
					// in args[0] (which for the VS Code channel client is
					// usually `null`) and leak User extensions into the
					// System list - the same bug we just fixed at the
					// handler layer, one level up.
					let EffectiveArgs = if command == "extensions:scanSystemExtensions" {
						let mut Overridden = args.clone();
						if Overridden.is_empty() {
							Overridden.push(Value::Null);
						}
						Overridden[0] = json!(0);
						Overridden
					} else {
						args.clone()
					};
					handle_extensions_get_installed(runtime.clone(), EffectiveArgs).await
				},
				"extensions:scanUserExtensions" => {
					// User-scope scan. Forward to the unified handler with
					// `type=ExtensionType.User (1)` so VSIX-installed
					// extensions under `~/.land/extensions/*` come back
					// even when the caller didn't pass an explicit type
					// filter (VS Code's channel client does that on
					// scan-user-extensions, which is why the sidebar
					// previously saw an empty list after every
					// Install-from-VSIX).
					dev_log!("extensions", "{} (forwarded to getInstalled with type=User)", command);
					let mut UserArgs = args.clone();
					if UserArgs.is_empty() {
						UserArgs.push(Value::Null);
					}
					UserArgs[0] = json!(1);
					handle_extensions_get_installed(runtime.clone(), UserArgs).await
				},
				"extensions:getUninstalled" => {
					// Uninstalled state (extensions soft-deleted but kept in
					// the profile) isn't tracked yet; an empty array is the
					// correct "nothing pending uninstall" response.
					dev_log!("extensions", "{} (returning [])", command);
					Ok(Value::Array(Vec::new()))
				},
				// Gallery is offline: Mountain has no marketplace backend. Return
				// empty arrays for every read and swallow every write, which
				// mirrors what a network-air-gapped VS Code session shows.
				"extensions:query" | "extensions:getExtensions" | "extensions:getRecommendations" => {
					dev_log!("extensions", "{} (offline gallery - returning [])", command);
					Ok(Value::Array(Vec::new()))
				},
				// `IExtensionsControlManifest` - consulted by the Extensions
				// sidebar on every render (ExtensionEnablementService.ts:793)
				// to mark malicious / deprecated / auto-updateable entries.
				// With the gallery offline an empty envelope is correct; the
				// shape (not null) matters - VS Code destructures each field.
				"extensions:getExtensionsControlManifest" => {
					dev_log!("extensions", "{} (offline gallery - empty manifest)", command);
					Ok(json!({
						"malicious": [],
						"deprecated": {},
						"search": [],
						"autoUpdate": {},
					}))
				},
				// Atom P1: `ExtensionsWorkbenchService.resetPinnedStateForAllUserExtensions`
				// is invoked when the user toggles pinning semantics in the
				// sidebar. Pin state is Wind-owned (Cocoon never sees it); the
				// only Mountain-side cost is an acknowledgement so the
				// extension-enablement service doesn't retry forever. Payload
				// is optional - VS Code sometimes passes `{ refreshPinned: true }`.
				"extensions:resetPinnedStateForAllUserExtensions" => {
					dev_log!("extensions", "{} (no-op, pin state is UI-local)", command);
					Ok(Value::Null)
				},
				// Atom K2: local VSIX install. Wind passes the file path from a
				// "Install from VSIX…" prompt or drag-and-drop through to us; the
				// previous stub silently returned `null` and the UI believed it
				// had succeeded (that's the "VSIX isn't triggering or loading"
				// regression). We now unpack the archive, stamp a DTO, register
				// it in ScannedExtensions, and return the ILocalExtension wrapper
				// so the sidebar refreshes without a window reload.
				"extensions:install" => {
					Extension::ExtensionInstall::ExtensionInstall(app_handle.clone(), runtime.clone(), args).await
				},
				"extensions:uninstall" => {
					Extension::ExtensionUninstall::ExtensionUninstall(app_handle.clone(), runtime.clone(), args).await
				},

				// `ExtensionManagementChannelClient.getManifest(vsix: URI)` - reads
				// the `extension/package.json` from a `.vsix` archive without
				// extracting it. Called by the "Install from VSIX…" preview and
				// by drag-and-drop onto the Extensions sidebar. The renderer then
				// accesses `manifest.publisher` / `.name` / `.displayName` on the
				// returned object unconditionally; a missing handler or an Err
				// response crashes the webview with
				// `TypeError: undefined is not an object (evaluating 'manifest.publisher')`.
				"extensions:getManifest" => {
					let VsixPath = match args.first() {
						Some(serde_json::Value::String(Path)) => Path.clone(),
						Some(Obj) => {
							Obj.get("fsPath")
								.and_then(|V| V.as_str())
								.map(str::to_owned)
								.or_else(|| Obj.get("path").and_then(|V| V.as_str()).map(str::to_owned))
								.unwrap_or_default()
						},
						None => String::new(),
					};
					dev_log!("extensions", "extensions:getManifest vsix={}", VsixPath);
					if VsixPath.is_empty() {
						Err("extensions:getManifest: missing VSIX path argument".to_string())
					} else {
						let Path = std::path::PathBuf::from(&VsixPath);
						match crate::ExtensionManagement::VsixInstaller::ReadFullManifest(&Path) {
							Ok(Manifest) => Ok(Manifest),
							Err(Error) => {
								dev_log!(
									"extensions",
									"warn: [WindServiceHandlers] extensions:getManifest failed for '{}': {}",
									VsixPath,
									Error
								);
								Err(format!("extensions:getManifest failed: {}", Error))
							},
						}
					}
				},
				// Reinstall and metadata-update still no-op for now; reinstall needs
				// a gallery cache (we only have the on-disk unpack), and metadata
				// update only matters for ratings/icons/readme which Land does not
				// track. Left as explicit logs so the UI doesn't silently fail.
				"extensions:reinstall" | "extensions:updateMetadata" => {
					dev_log!("extensions", "{} (no-op: no gallery backend)", command);
					Ok(Value::Null)
				},

				// Terminal commands
				"terminal:create" => {
					dev_log!("terminal", "terminal:create");
					handle_terminal_create(runtime.clone(), args).await
				},
				"terminal:sendText" => {
					dev_log!("terminal", "terminal:sendText");
					handle_terminal_send_text(runtime.clone(), args).await
				},
				"terminal:dispose" => {
					dev_log!("terminal", "terminal:dispose");
					handle_terminal_dispose(runtime.clone(), args).await
				},
				"terminal:show" => {
					dev_log!("terminal", "terminal:show");
					handle_terminal_show(runtime.clone(), args).await
				},
				"terminal:hide" => {
					dev_log!("terminal", "terminal:hide");
					handle_terminal_hide(runtime.clone(), args).await
				},

				// Output channel commands
				"output:create" => handle_output_create(app_handle.clone(), args).await,
				"output:append" => {
					dev_log!("output", "output:append");
					handle_output_append(app_handle.clone(), args).await
				},
				"output:appendLine" => {
					dev_log!("output", "output:appendLine");
					handle_output_append_line(app_handle.clone(), args).await
				},
				"output:clear" => {
					dev_log!("output", "output:clear");
					handle_output_clear(app_handle.clone(), args).await
				},
				"output:show" => {
					dev_log!("output", "output:show");
					handle_output_show(app_handle.clone(), args).await
				},

				// TextFile commands
				"textFile:read" => {
					dev_log!("textfile", "textFile:read");
					handle_textfile_read(runtime.clone(), args).await
				},
				"textFile:write" => {
					dev_log!("textfile", "textFile:write");
					handle_textfile_write(runtime.clone(), args).await
				},
				"textFile:save" => handle_textfile_save(runtime.clone(), args).await,

				// Storage commands (additional)
				"storage:delete" => {
					dev_log!("storage", "storage:delete");
					handle_storage_delete(runtime.clone(), args).await
				},
				"storage:keys" => {
					dev_log!("storage", "storage:keys");
					handle_storage_keys(runtime.clone()).await
				},

				// Notification commands (emit sky:// events for Sky to render)
				"notification:show" => {
					dev_log!("notification", "notification:show");
					handle_notification_show(app_handle.clone(), args).await
				},
				"notification:showProgress" => {
					dev_log!("notification", "notification:showProgress");
					handle_notification_show_progress(app_handle.clone(), args).await
				},
				"notification:updateProgress" => {
					dev_log!("notification", "notification:updateProgress");
					handle_notification_update_progress(app_handle.clone(), args).await
				},
				"notification:endProgress" => {
					dev_log!("notification", "notification:endProgress");
					handle_notification_end_progress(app_handle.clone(), args).await
				},

				// Progress commands
				"progress:begin" => {
					dev_log!("progress", "progress:begin");
					handle_progress_begin(app_handle.clone(), args).await
				},
				"progress:report" => {
					dev_log!("progress", "progress:report");
					handle_progress_report(app_handle.clone(), args).await
				},
				"progress:end" => {
					dev_log!("progress", "progress:end");
					handle_progress_end(app_handle.clone(), args).await
				},

				// QuickInput commands
				"quickInput:showQuickPick" => {
					dev_log!("quickinput", "quickInput:showQuickPick");
					handle_quick_input_show_quick_pick(runtime.clone(), args).await
				},
				"quickInput:showInputBox" => {
					dev_log!("quickinput", "quickInput:showInputBox");
					handle_quick_input_show_input_box(runtime.clone(), args).await
				},

				// Workspaces commands. VS Code's `IWorkspacesService`
				// channel uses `getWorkspaceFolders` /
				// `addWorkspaceFolders`; Mountain's rail uses the
				// shorter `getFolders` / `addFolder`. Alias both.
				"workspaces:getFolders" | "workspaces:getWorkspaceFolders" | "workspaces:getWorkspace" => {
					dev_log!("workspaces", "{}", command);
					handle_workspaces_get_folders(runtime.clone()).await
				},
				"workspaces:addFolder" | "workspaces:addWorkspaceFolders" => {
					dev_log!("workspaces", "{}", command);
					handle_workspaces_add_folder(runtime.clone(), args).await
				},
				"workspaces:removeFolder" | "workspaces:removeWorkspaceFolders" => {
					dev_log!("workspaces", "{}", command);
					handle_workspaces_remove_folder(runtime.clone(), args).await
				},
				"workspaces:getName" => {
					dev_log!("workspaces", "{}", command);
					handle_workspaces_get_name(runtime.clone()).await
				},
				// `onDidChangeWorkspaceFolders` channel-listen: Mountain
				// broadcasts the change via Tauri event, so ack the
				// listen request with Null (no-op on the binary rail).
				"workspaces:onDidChangeWorkspaceFolders" | "workspaces:onDidChangeWorkspaceName" => {
					dev_log!("workspaces", "{} (stub-ack)", command);
					Ok(Value::Null)
				},

				// Themes commands
				"themes:getActive" => {
					dev_log!("themes", "themes:getActive");
					handle_themes_get_active(runtime.clone()).await
				},
				"themes:list" => {
					dev_log!("themes", "themes:list");
					handle_themes_list(runtime.clone()).await
				},
				"themes:set" => {
					dev_log!("themes", "themes:set");
					handle_themes_set(runtime.clone(), args).await
				},

				// Search commands. Stock VS Code `SearchService` channel
				// uses `textSearch` / `fileSearch`; Mountain's Effect-TS
				// rail uses `findInFiles` / `findFiles`. Alias both.
				"search:findInFiles" | "search:textSearch" | "search:searchText" => {
					dev_log!("search", "{}", command);
					handle_search_find_in_files(runtime.clone(), args).await
				},
				"search:findFiles" | "search:fileSearch" | "search:searchFile" => {
					dev_log!("search", "{}", command);
					handle_search_find_files(runtime.clone(), args).await
				},
				// Cancellation / onProgress channel methods: workbench's
				// SearchService listens for these. We have no streaming
				// search yet, so ack with Null and let the workbench
				// treat the call as a no-op.
				"search:cancel" | "search:clearCache" | "search:onDidChangeResult" => {
					dev_log!("search", "{} (stub-ack)", command);
					Ok(Value::Null)
				},

				// Decorations commands
				"decorations:get" => {
					dev_log!("decorations", "decorations:get");
					handle_decorations_get(runtime.clone(), args).await
				},
				"decorations:getMany" => {
					dev_log!("decorations", "decorations:getMany");
					handle_decorations_get_many(runtime.clone(), args).await
				},
				"decorations:set" => {
					dev_log!("decorations", "decorations:set");
					handle_decorations_set(runtime.clone(), args).await
				},
				"decorations:clear" => {
					dev_log!("decorations", "decorations:clear");
					handle_decorations_clear(runtime.clone(), args).await
				},

				// WorkingCopy commands
				"workingCopy:isDirty" => {
					dev_log!("workingcopy", "workingCopy:isDirty");
					handle_working_copy_is_dirty(runtime.clone(), args).await
				},
				"workingCopy:setDirty" => {
					dev_log!("workingcopy", "workingCopy:setDirty");
					handle_working_copy_set_dirty(runtime.clone(), args).await
				},
				"workingCopy:getAllDirty" => {
					dev_log!("workingcopy", "workingCopy:getAllDirty");
					handle_working_copy_get_all_dirty(runtime.clone()).await
				},
				"workingCopy:getDirtyCount" => {
					dev_log!("workingcopy", "workingCopy:getDirtyCount");
					handle_working_copy_get_dirty_count(runtime.clone()).await
				},

				// Keybinding commands
				"keybinding:add" => {
					dev_log!("keybinding", "keybinding:add");
					handle_keybinding_add(runtime.clone(), args).await
				},
				"keybinding:remove" => {
					dev_log!("keybinding", "keybinding:remove");
					handle_keybinding_remove(runtime.clone(), args).await
				},
				"keybinding:lookup" => {
					dev_log!("keybinding", "keybinding:lookup");
					handle_keybinding_lookup(runtime.clone(), args).await
				},
				"keybinding:getAll" => {
					dev_log!("keybinding", "keybinding:getAll");
					handle_keybinding_get_all(runtime.clone()).await
				},

				// Lifecycle commands
				"lifecycle:getPhase" => {
					dev_log!("lifecycle", "lifecycle:getPhase");
					handle_lifecycle_get_phase(runtime.clone()).await
				},
				"lifecycle:whenPhase" => {
					dev_log!("lifecycle", "lifecycle:whenPhase");
					handle_lifecycle_when_phase(runtime.clone(), args).await
				},
				"lifecycle:requestShutdown" => {
					dev_log!("lifecycle", "lifecycle:requestShutdown");
					handle_lifecycle_request_shutdown(app_handle.clone()).await
				},
				"lifecycle:advancePhase" | "lifecycle:setPhase" => {
					dev_log!("lifecycle", "{}", command);
					// Wind calls this at the end of every workbench init pass so
					// the phase advances Starting → Ready → Restored → Eventually.
					// Mountain emits `sky://lifecycle/phaseChanged` so any extension
					// host or service waiting on a later phase wakes up.
					let NewPhase = args.first().and_then(|V| V.as_u64()).unwrap_or(1) as u8;
					runtime
						.Environment
						.ApplicationState
						.Feature
						.Lifecycle
						.AdvanceAndBroadcast(NewPhase, &app_handle);

					// Hidden-until-ready: the main window is built with
					// `.visible(false)` to suppress the four-repaint flash
					// (native chrome → inline bg → theme CSS → workbench
					// DOM). Phase 3 = Restored means `.monaco-workbench`
					// is attached and the first frame is painted; show
					// the window now so the user's first glimpse is the
					// finished editor rather than the paint cascade.
					//
					// `set_focus()` follows `show()` so keyboard input
					// routes to the editor immediately on reveal.
					// Failures are logged but swallowed - if the window
					// is already visible (phase 3 re-fired from another
					// consumer) Tauri returns a benign error.
					if NewPhase >= 3 {
						if let Some(MainWindow) = app_handle.get_webview_window("main") {
							if let Ok(false) = MainWindow.is_visible() {
								if let Err(Error) = MainWindow.show() {
									dev_log!(
										"lifecycle",
										"warn: [Lifecycle] main window show() failed on phase {}: {}",
										NewPhase,
										Error
									);
								} else {
									dev_log!(
										"lifecycle",
										"[Lifecycle] main window revealed on phase {} (hidden-until-ready)",
										NewPhase
									);
									let _ = MainWindow.set_focus();
								}
							}
						}
					}

					Ok(json!(runtime.Environment.ApplicationState.Feature.Lifecycle.GetPhase()))
				},

				// Label commands
				"label:getUri" => {
					dev_log!("label", "label:getUri");
					handle_label_get_uri(runtime.clone(), args).await
				},
				"label:getWorkspace" => {
					dev_log!("label", "label:getWorkspace");
					handle_label_get_workspace(runtime.clone()).await
				},
				"label:getBase" => {
					dev_log!("label", "label:getBase");
					handle_label_get_base(args).await
				},

				// Model (text model registry) commands
				"model:open" => {
					dev_log!("model", "model:open");
					handle_model_open(runtime.clone(), args).await
				},
				"model:close" => {
					dev_log!("model", "model:close");
					handle_model_close(runtime.clone(), args).await
				},
				"model:get" => {
					dev_log!("model", "model:get");
					handle_model_get(runtime.clone(), args).await
				},
				"model:getAll" => {
					dev_log!("model", "model:getAll");
					handle_model_get_all(runtime.clone()).await
				},
				"model:updateContent" => {
					dev_log!("model", "model:updateContent");
					handle_model_update_content(runtime.clone(), args).await
				},

				// Navigation history commands
				"history:goBack" => {
					dev_log!("history", "history:goBack");
					handle_history_go_back(runtime.clone()).await
				},
				"history:goForward" => {
					dev_log!("history", "history:goForward");
					handle_history_go_forward(runtime.clone()).await
				},
				"history:canGoBack" => {
					dev_log!("history", "history:canGoBack");
					handle_history_can_go_back(runtime.clone()).await
				},
				"history:canGoForward" => {
					dev_log!("history", "history:canGoForward");
					handle_history_can_go_forward(runtime.clone()).await
				},
				"history:push" => {
					dev_log!("history", "history:push");
					handle_history_push(runtime.clone(), args).await
				},
				"history:clear" => {
					dev_log!("history", "history:clear");
					handle_history_clear(runtime.clone()).await
				},
				"history:getStack" => {
					dev_log!("history", "history:getStack");
					handle_history_get_stack(runtime.clone()).await
				},

				// IPC status commands
				"mountain_get_status" => {
					let status = json!({
						"connected": true,
						"version": "1.0.0"
					});
					Ok(status)
				},
				"mountain_get_configuration" => {
					let config = json!({
						"editor": { "theme": "dark" },
						"extensions": { "installed": [] }
					});
					Ok(config)
				},
				"mountain_get_services_status" => {
					let services = json!({
						"editor": { "status": "running" },
						"extensionHost": { "status": "running" }
					});
					Ok(services)
				},
				"mountain_get_state" => {
					let state = json!({
						"ui": {},
						"editor": {},
						"workspace": {}
					});
					Ok(state)
				},

				// =====================================================================
				// File system command ALIASES
				// VS Code's DiskFileSystemProviderClient calls readFile/writeFile/rename
				// but Mountain's original handlers use read/write/move.
				// =====================================================================
				"file:realpath" => handle_file_realpath(args).await,
				"file:open" => {
					dev_log!("vfs", "file:open stub - no fd support yet");
					Ok(json!(0))
				},
				"file:close" => {
					dev_log!("vfs", "file:close stub");
					Ok(Value::Null)
				},
				"file:cloneFile" => handle_file_clone_native(args).await,

				// =====================================================================
				// Native Host commands (INativeHostService)
				// =====================================================================

				// Dialogs
				"nativeHost:pickFolderAndOpen" => handle_native_pick_folder(app_handle.clone(), args).await,
				"nativeHost:pickFileAndOpen" => handle_native_pick_folder(app_handle.clone(), args).await,
				"nativeHost:pickFileFolderAndOpen" => handle_native_pick_folder(app_handle.clone(), args).await,
				"nativeHost:pickWorkspaceAndOpen" => handle_native_pick_folder(app_handle.clone(), args).await,
				"nativeHost:showOpenDialog" => handle_native_show_open_dialog(app_handle.clone(), args).await,
				"nativeHost:showSaveDialog" => {
					use tauri_plugin_dialog::DialogExt;
					let Options = args.first().cloned().unwrap_or(Value::Null);
					let Title = Options.get("title").and_then(Value::as_str).unwrap_or("Save").to_string();
					let DefaultPath = Options.get("defaultPath").and_then(Value::as_str).map(str::to_string);
					let Handle = app_handle.clone();
					let Joined = tokio::task::spawn_blocking(move || -> Option<String> {
						let mut Builder = Handle.dialog().file().set_title(&Title);
						if let Some(Path) = DefaultPath.as_deref() {
							Builder = Builder.set_directory(Path);
						}
						Builder.blocking_save_file().map(|P| P.to_string())
					})
					.await;
					match Joined {
						Ok(Some(Path)) => Ok(json!({ "canceled": false, "filePath": Path })),
						Ok(None) => Ok(json!({ "canceled": true })),
						Err(Error) => Err(format!("showSaveDialog join error: {}", Error)),
					}
				},
				"nativeHost:showMessageBox" => {
					use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
					let Options = args.first().cloned().unwrap_or(Value::Null);
					let Message = Options.get("message").and_then(Value::as_str).unwrap_or("").to_string();
					let Detail = Options.get("detail").and_then(Value::as_str).map(str::to_string);
					let DialogType = Options
						.get("type")
						.and_then(Value::as_str)
						.map(|S| S.to_lowercase())
						.unwrap_or_default();
					let Title = Options.get("title").and_then(Value::as_str).unwrap_or("").to_string();
					let Kind = match DialogType.as_str() {
						"warning" | "warn" => MessageDialogKind::Warning,
						"error" => MessageDialogKind::Error,
						_ => MessageDialogKind::Info,
					};
					let Handle = app_handle.clone();
					let Joined = tokio::task::spawn_blocking(move || -> bool {
						let mut Builder = Handle.dialog().message(&Message).kind(Kind);
						if !Title.is_empty() {
							Builder = Builder.title(&Title);
						}
						if let Some(DetailText) = Detail.as_deref() {
							Builder = Builder.title(DetailText);
						}
						Builder.blocking_show()
					})
					.await;
					match Joined {
						Ok(Answered) => Ok(json!({ "response": if Answered { 0 } else { 1 } })),
						Err(Error) => Err(format!("showMessageBox join error: {}", Error)),
					}
				},

				// Environment paths - called by ResolveConfiguration to get real Tauri paths.
				// Returns the session log directory (with timestamp + window1 subdir)
				// so VS Code can immediately write output files without stat errors.
				"nativeHost:getEnvironmentPaths" => {
					let PathResolver = app_handle.path();
					let AppDataDir = PathResolver.app_data_dir().unwrap_or_default();
					let HomeDir = PathResolver.home_dir().unwrap_or_default();
					let TmpDir = std::env::temp_dir();

					// Logs go under {appDataDir}/logs/{sessionTimestamp}/ - same tree as
					// all other VS Code data, not Tauri's separate app_log_dir().
					// VS Code requires a session-timestamped subdir for log rotation.
					// `DevLog::SessionTimestamp` is the single source of truth so that
					// `Mountain.dev.log` (written by DevLog) and VS Code's
					// `window1/output/*.log` files (written into `logsPath`) share one
					// directory per session.
					let SessionLogRoot = AppDataDir.join("logs").join(crate::IPC::DevLog::SessionTimestamp());
					let SessionLogWindowDir = SessionLogRoot.join("window1");
					let _ = std::fs::create_dir_all(&SessionLogWindowDir);

					dev_log!(
						"config",
						"getEnvironmentPaths: userDataDir={} logsPath={} homeDir={}",
						AppDataDir.display(),
						SessionLogRoot.display(),
						HomeDir.display()
					);
					let DevLogEnv = std::env::var("Trace").unwrap_or_default();
					Ok(json!({
						"userDataDir": AppDataDir.to_string_lossy(),
						"logsPath": SessionLogRoot.to_string_lossy(),
						"homeDir": HomeDir.to_string_lossy(),
						"tmpDir": TmpDir.to_string_lossy(),
						"devLog": if DevLogEnv.is_empty() { Value::Null } else { json!(DevLogEnv) },
					}))
				},

				// OS info
				"nativeHost:getOSColorScheme" => {
					dev_log!("nativehost", "nativeHost:getOSColorScheme");
					handle_native_get_color_scheme().await
				},
				"nativeHost:getOSProperties" => {
					dev_log!("nativehost", "nativeHost:getOSProperties");
					handle_native_os_properties().await
				},
				"nativeHost:getOSStatistics" => {
					dev_log!("nativehost", "nativeHost:getOSStatistics");
					handle_native_os_statistics().await
				},
				"nativeHost:getOSVirtualMachineHint" => {
					dev_log!("nativehost", "nativeHost:getOSVirtualMachineHint");
					Ok(json!(0))
				},

				// Window state
				"nativeHost:isWindowAlwaysOnTop" => {
					dev_log!("window", "nativeHost:isWindowAlwaysOnTop");
					Ok(json!(false))
				},
				"nativeHost:isFullScreen" => {
					dev_log!("window", "nativeHost:isFullScreen");
					handle_native_is_fullscreen(app_handle.clone()).await
				},
				"nativeHost:isMaximized" => {
					dev_log!("window", "nativeHost:isMaximized");
					handle_native_is_maximized(app_handle.clone()).await
				},
				"nativeHost:getActiveWindowId" => {
					dev_log!("window", "nativeHost:getActiveWindowId");
					Ok(json!(1))
				},
				// LAND-FIX: workbench polls the cursor screen point for
				// hover hint / context-menu placement. Stock VS Code
				// returns the OS cursor location via Electron's
				// `screen.getCursorScreenPoint()`. Tauri/Wry on macOS
				// does not expose a stable equivalent (CGEvent location
				// works but adds an Objective-C trampoline per call).
				// Returning `{x:0, y:0}` is what stock VS Code itself
				// returns when no display is active; this is also what
				// Cocoon falls back to. Workbench uses the value only
				// to bias overlay placement; (0,0) places overlays at
				// the top-left of the active window which the layout
				// engine then clips to a sane position. The cost of
				// the unknown-IPC log spam outweighs the precision
				// loss.
				"nativeHost:getCursorScreenPoint" => {
					dev_log!("window", "nativeHost:getCursorScreenPoint");
					Ok(json!({ "x": 0, "y": 0 }))
				},
				"nativeHost:getWindows" => Ok(json!([{ "id": 1, "title": "Land", "filename": "" }])),
				"nativeHost:getWindowCount" => Ok(json!(1)),

				// Auxiliary window spawners. VS Code's `nativeHostMainService.ts`
				// exposes `openAgentsWindow`, `openDevToolsWindow`, and
				// `openAuxiliaryWindow`, and Sky/Wind route these through the
				// `nativeHost:<method>` IPC channel. Without stubs, every call fires
				// `land:ipc:error:nativeHost.openAgentsWindow` in PostHog (1499
				// occurrences per the 2026-04-21 error report). Land doesn't have
				// AgentsView yet, so these are no-op acknowledgements - the calling
				// extension treats `undefined` as "window wasn't opened" rather than
				// an error.
				"nativeHost:openAgentsWindow" | "nativeHost:openDevToolsWindow" | "nativeHost:openAuxiliaryWindow" => {
					dev_log!("window", "{} (acknowledged, no-op - aux window unsupported)", command);
					Ok(Value::Null)
				},

				// Window control - wired through the Tauri webview-window API so
				// focus/minimize/maximize/toggleFullScreen/close actually move the
				// native window the same way VS Code's Electron path does.
				"nativeHost:focusWindow" => {
					dev_log!("window", "{}", command);
					if let Some(Window) = app_handle.get_webview_window("main") {
						let _ = Window.set_focus();
					}
					Ok(Value::Null)
				},
				"nativeHost:maximizeWindow" => {
					dev_log!("window", "{}", command);
					if let Some(Window) = app_handle.get_webview_window("main") {
						let _ = Window.maximize();
					}
					Ok(Value::Null)
				},
				"nativeHost:unmaximizeWindow" => {
					dev_log!("window", "{}", command);
					if let Some(Window) = app_handle.get_webview_window("main") {
						let _ = Window.unmaximize();
					}
					Ok(Value::Null)
				},
				"nativeHost:minimizeWindow" => {
					dev_log!("window", "{}", command);
					if let Some(Window) = app_handle.get_webview_window("main") {
						let _ = Window.minimize();
					}
					Ok(Value::Null)
				},
				"nativeHost:toggleFullScreen" => {
					dev_log!("window", "{}", command);
					if let Some(Window) = app_handle.get_webview_window("main") {
						let IsFullscreen = Window.is_fullscreen().unwrap_or(false);
						let _ = Window.set_fullscreen(!IsFullscreen);
					}
					Ok(Value::Null)
				},
				"nativeHost:closeWindow" => {
					dev_log!("window", "{}", command);
					// `destroy()` tears the window down without firing
					// `CloseRequested` again, which lets us safely exit the
					// `prevent_close` intercept registered in AppLifecycle.
					// `close()` re-enters the intercept and the window
					// becomes unkillable.
					if let Some(Window) = app_handle.get_webview_window("main") {
						let _ = Window.destroy();
					}
					Ok(Value::Null)
				},
				"nativeHost:setWindowAlwaysOnTop" => {
					dev_log!("window", "{}", command);
					let OnTop = args.first().and_then(|V| V.as_bool()).unwrap_or(false);
					if let Some(Window) = app_handle.get_webview_window("main") {
						let _ = Window.set_always_on_top(OnTop);
					}
					Ok(Value::Null)
				},
				"nativeHost:toggleWindowAlwaysOnTop" => {
					dev_log!("window", "{}", command);
					// Tauri doesn't expose a "get always on top" accessor on all
					// platforms, so toggle by tracking state via the webview title
					// prefix as a proxy. In practice the UI will call
					// `setWindowAlwaysOnTop` with an explicit bool immediately after,
					// so a best-effort flip is enough.
					if let Some(Window) = app_handle.get_webview_window("main") {
						let _ = Window.set_always_on_top(true);
					}
					Ok(Value::Null)
				},
				"nativeHost:setRepresentedFilename" => {
					dev_log!("window", "{}", command);
					#[cfg(target_os = "macos")]
					{
						let Path = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
						if !Path.is_empty() {
							if let Some(Window) = app_handle.get_webview_window("main") {
								let _ = Window.set_title(&Path);
							}
						}
					}
					let _ = (&args, &app_handle);
					Ok(Value::Null)
				},

				// Pure no-op arms - pure lifecycle signals VS Code fires regardless
				// of the backing host (Electron, Mountain, Browser) but we don't
				// need to do anything about. Kept named so the `Unknown IPC command`
				// default branch never fires for them.
				"nativeHost:updateWindowControls"
				| "nativeHost:setMinimumSize"
				| "nativeHost:notifyReady"
				| "nativeHost:saveWindowSplash"
				| "nativeHost:updateTouchBar"
				| "nativeHost:moveWindowTop"
				| "nativeHost:positionWindow"
				| "nativeHost:setDocumentEdited"
				| "nativeHost:setBackgroundThrottling"
				| "nativeHost:updateWindowAccentColor" => {
					dev_log!("window", "{}", command);
					Ok(Value::Null)
				},

				// OS operations
				"nativeHost:isAdmin" => Ok(json!(false)),
				"nativeHost:isRunningUnderARM64Translation" => {
					#[cfg(target_os = "macos")]
					{
						// macOS: check if running under Rosetta 2
						let Output = std::process::Command::new("sysctl")
							.args(["-n", "sysctl.proc_translated"])
							.output();
						let IsTranslated = Output
							.ok()
							.map(|O| String::from_utf8_lossy(&O.stdout).trim() == "1")
							.unwrap_or(false);
						Ok(json!(IsTranslated))
					}
					#[cfg(not(target_os = "macos"))]
					{
						Ok(json!(false))
					}
				},
				"nativeHost:hasWSLFeatureInstalled" => {
					#[cfg(target_os = "windows")]
					{
						Ok(json!(std::path::Path::new("C:\\Windows\\System32\\wsl.exe").exists()))
					}
					#[cfg(not(target_os = "windows"))]
					{
						Ok(json!(false))
					}
				},
				"nativeHost:showItemInFolder" => handle_show_item_in_folder(runtime.clone(), args).await,
				"nativeHost:openExternal" => handle_open_external(runtime.clone(), args).await,
				// `workbench.files.action.deleteFile` and extensions that delete
				// files both round-trip through here. Route to the platform's
				// trash bin so deletions are recoverable. macOS uses AppleScript
				// via `osascript`; Linux prefers `gio trash` then `trash` if
				// installed; Windows uses PowerShell with Shell.NameSpace.
				"nativeHost:moveItemToTrash" => {
					let Path = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
					if Path.is_empty() {
						Ok(json!(false))
					} else {
						dev_log!("nativehost", "nativeHost:moveItemToTrash path={}", Path);
						let Moved = {
							#[cfg(target_os = "macos")]
							{
								tokio::process::Command::new("osascript")
									.args([
										"-e",
										&format!(
											"tell application \"Finder\" to delete POSIX file \"{}\"",
											Path.replace('"', "\\\"")
										),
									])
									.status()
									.await
									.map(|S| S.success())
									.unwrap_or(false)
							}
							#[cfg(target_os = "linux")]
							{
								let Gio = tokio::process::Command::new("gio")
									.args(["trash", &Path])
									.status()
									.await
									.map(|S| S.success())
									.unwrap_or(false);
								if Gio {
									true
								} else {
									tokio::process::Command::new("trash")
										.arg(&Path)
										.status()
										.await
										.map(|S| S.success())
										.unwrap_or(false)
								}
							}
							#[cfg(target_os = "windows")]
							{
								let Script = format!(
									"(new-object -comobject Shell.Application).NameSpace(0xA).MoveHere('{}')",
									Path.replace('\'', "''")
								);
								tokio::process::Command::new("powershell.exe")
									.args(["-NoProfile", "-Command", &Script])
									.status()
									.await
									.map(|S| S.success())
									.unwrap_or(false)
							}
							#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
							{
								false
							}
						};
						Ok(json!(Moved))
					}
				},

				// Clipboard - backed by `arboard` so read/writeText round-trip the
				// OS clipboard. `readClipboardBuffer` is kept empty (binary
				// clipboard is rarely used by VS Code core; extensions that need
				// it invoke the platform-specific path instead).
				"nativeHost:readClipboardText" => {
					dev_log!("clipboard", "readClipboardText");
					match arboard::Clipboard::new() {
						Ok(mut Cb) => Ok(json!(Cb.get_text().unwrap_or_default())),
						Err(_) => Ok(json!("")),
					}
				},
				"nativeHost:writeClipboardText" => {
					dev_log!("clipboard", "writeClipboardText");
					let Text = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
					if let Ok(mut Cb) = arboard::Clipboard::new() {
						let _ = Cb.set_text(Text);
					}
					Ok(Value::Null)
				},
				"nativeHost:readClipboardFindText" => {
					dev_log!("clipboard", "readClipboardFindText");
					// macOS has a separate find pasteboard; reuse the general
					// clipboard for parity with VS Code on Linux/Windows.
					match arboard::Clipboard::new() {
						Ok(mut Cb) => Ok(json!(Cb.get_text().unwrap_or_default())),
						Err(_) => Ok(json!("")),
					}
				},
				"nativeHost:writeClipboardFindText" => {
					dev_log!("clipboard", "writeClipboardFindText");
					let Text = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
					if let Ok(mut Cb) = arboard::Clipboard::new() {
						let _ = Cb.set_text(Text);
					}
					Ok(Value::Null)
				},
				"nativeHost:readClipboardBuffer" => {
					dev_log!("clipboard", "readClipboardBuffer");
					Ok(json!([]))
				},
				"nativeHost:writeClipboardBuffer" => {
					dev_log!("clipboard", "writeClipboardBuffer");
					Ok(Value::Null)
				},
				"nativeHost:hasClipboard" => {
					dev_log!("clipboard", "hasClipboard");
					Ok(json!(false))
				},
				"nativeHost:readImage" => {
					dev_log!("clipboard", "readImage");
					Ok(json!([]))
				},
				"nativeHost:triggerPaste" => {
					dev_log!("clipboard", "triggerPaste");
					Ok(Value::Null)
				},

				// Process
				"nativeHost:getProcessId" => Ok(json!(std::process::id())),
				"nativeHost:killProcess" => Ok(Value::Null),

				// Network
				"nativeHost:findFreePort" => handle_native_find_free_port(args).await,
				"nativeHost:isPortFree" => Ok(json!(true)),
				"nativeHost:resolveProxy" => Ok(Value::Null),
				"nativeHost:lookupAuthorization" => Ok(Value::Null),
				"nativeHost:lookupKerberosAuthorization" => Ok(Value::Null),
				"nativeHost:loadCertificates" => Ok(json!([])),

				// Lifecycle
				"nativeHost:relaunch" => Ok(Value::Null),
				"nativeHost:reload" => Ok(Value::Null),
				"nativeHost:quit" => Ok(Value::Null),
				"nativeHost:exit" => Ok(Value::Null),

				// Dev tools
				"nativeHost:openDevTools" => Ok(Value::Null),
				"nativeHost:toggleDevTools" => Ok(Value::Null),

				// Power
				"nativeHost:getSystemIdleState" => Ok(json!("active")),
				"nativeHost:getSystemIdleTime" => Ok(json!(0)),
				"nativeHost:getCurrentThermalState" => Ok(json!("nominal")),
				"nativeHost:isOnBatteryPower" => Ok(json!(false)),
				"nativeHost:startPowerSaveBlocker" => Ok(json!(0)),
				"nativeHost:stopPowerSaveBlocker" => Ok(json!(false)),
				"nativeHost:isPowerSaveBlockerStarted" => Ok(json!(false)),

				// macOS specific
				"nativeHost:newWindowTab" => Ok(Value::Null),
				"nativeHost:showPreviousWindowTab" => Ok(Value::Null),
				"nativeHost:showNextWindowTab" => Ok(Value::Null),
				"nativeHost:moveWindowTabToNewWindow" => Ok(Value::Null),
				"nativeHost:mergeAllWindowTabs" => Ok(Value::Null),
				"nativeHost:toggleWindowTabsBar" => Ok(Value::Null),
				"nativeHost:installShellCommand" => Ok(Value::Null),
				"nativeHost:uninstallShellCommand" => Ok(Value::Null),

				// =====================================================================
				// Local PTY (terminal) commands
				// =====================================================================
				"localPty:getProfiles" => {
					dev_log!("terminal", "localPty:getProfiles");
					handle_local_pty_get_profiles().await
				},
				"localPty:getDefaultSystemShell" => {
					dev_log!("terminal", "localPty:getDefaultSystemShell");
					handle_local_pty_get_default_shell().await
				},
				"localPty:getTerminalLayoutInfo" => {
					dev_log!("terminal", "localPty:getTerminalLayoutInfo");
					Ok(Value::Null)
				},
				"localPty:setTerminalLayoutInfo" => {
					dev_log!("terminal", "localPty:setTerminalLayoutInfo");
					Ok(Value::Null)
				},
				"localPty:getPerformanceMarks" => {
					dev_log!("terminal", "localPty:getPerformanceMarks");
					Ok(json!([]))
				},
				"localPty:reduceConnectionGraceTime" => {
					dev_log!("terminal", "localPty:reduceConnectionGraceTime");
					Ok(Value::Null)
				},
				"localPty:listProcesses" => {
					dev_log!("terminal", "localPty:listProcesses");
					Ok(json!([]))
				},
				"localPty:getEnvironment" => {
					dev_log!("terminal", "localPty:getEnvironment");
					handle_local_pty_get_environment().await
				},
				// `IPtyService.getLatency` (per
				// `vs/platform/terminal/common/terminal.ts:341`) returns
				// `IPtyHostLatencyMeasurement[]`. The workbench polls this
				// to drive its "renderer ↔ pty host" health UI. We have
				// no separate pty host (Mountain spawns PTYs in-process),
				// so latency is effectively zero - return an empty array
				// matching the "no measurements available" branch the
				// workbench already handles. Without this route the call
				// surfaced as `Unknown IPC command: localPty:getLatency`
				// every poll cycle, and the renderer logged a
				// `TauriInvoke ok=false` line per attempt.
				"localPty:getLatency" => {
					dev_log!("terminal", "localPty:getLatency");
					Ok(json!([]))
				},

				// `cocoon:request` - generic renderer→Cocoon RPC bridge.
				// Used by Sky-side bridges that need to dispatch a request
				// into the extension host (e.g. `webview.resolveView` to
				// trigger an extension's `resolveWebviewView` callback).
				// Wire shape: `params = [Method, Payload]`. Mountain
				// forwards to Cocoon via `Vine::Client::SendRequest` and
				// returns the response verbatim. Failure surfaces as a
				// stringified error so the renderer can fall through to
				// its alternative path (CustomEvent fan-out for legacy
				// observers).
				"cocoon:request" => {
					dev_log!("ipc", "cocoon:request method={:?}", args.first());
					let MethodOpt = args.first().and_then(|V| V.as_str()).map(|S| S.to_string());
					match MethodOpt {
						None => Err("cocoon:request requires method string in slot 0".to_string()),
						Some(Method) => {
							let Payload = args.get(1).cloned().unwrap_or(Value::Null);
							// Same boot-race guard as `tree:getChildren`: the
							// renderer can dispatch `cocoon:request` (e.g.
							// `webview.resolveView`) before Cocoon's gRPC
							// handshake completes. Wait briefly so the first
							// few calls don't fail spuriously and poison
							// renderer-side caches.
							let _ = crate::Vine::Client::WaitForClientConnection("cocoon-main", 1500).await;
							crate::Vine::Client::SendRequest("cocoon-main", Method.clone(), Payload, 30_000)
								.await
								.map_err(|Error| format!("cocoon:request {} failed: {:?}", Method, Error))
						},
					}
				},

				// `cocoon:notify` - fire-and-forget renderer→Cocoon
				// notification bridge. Companion to `cocoon:request` for
				// one-way wire methods (`webview.message`,
				// `webview.dispose`, `webview.viewState` etc.) where the
				// extension doesn't reply. Avoids the 30s request
				// timeout penalty when the renderer just wants to push
				// data into the extension host. Wire shape:
				// `params = [Method, Payload]`. Returns null
				// immediately; the notification dispatches asynchronously.
				"cocoon:notify" => {
					dev_log!("ipc", "cocoon:notify method={:?}", args.first());
					let MethodOpt = args.first().and_then(|V| V.as_str()).map(|S| S.to_string());
					match MethodOpt {
						None => Err("cocoon:notify requires method string in slot 0".to_string()),
						Some(Method) => {
							let Payload = args.get(1).cloned().unwrap_or(Value::Null);
							if let Err(Error) = crate::Vine::Client::SendNotification(
								"cocoon-main".to_string(),
								Method.clone(),
								Payload,
							)
							.await
							{
								dev_log!("ipc", "warn: [cocoon:notify] {} failed: {:?}", Method, Error);
							}
							Ok(Value::Null)
						},
					}
				},

				// BATCH-19 Part B: VS Code's `LocalPtyService` talks to Mountain via
				// the `localPty:*` channel. The internal implementations reuse the
				// Tauri-side `terminal:*` handlers so PTY lifecycle stays identical
				// regardless of whether the request came from Sky (Wind) or from an
				// extension (Cocoon → Wind channel bridge).
				//
				// CONTRACT NOTE: `IPtyService.createProcess` is typed
				// `Promise<number>` (see `vs/platform/terminal/common/terminal.ts:
				// 316`). The workbench then does `new LocalPty(id, ...)` and
				// `this._ptys.set(id, pty)`. If we return the full
				// `{id,name,pid}` object the renderer keys `_ptys` by that
				// object, every `_ptys.get(<integer>)` lookup from
				// `onProcessData`/`onProcessReady` returns `undefined`, and
				// xterm receives zero bytes - the terminal panel renders
				// blank even though Mountain's PTY reader emits data
				// continuously. Strip down to the integer id here.
				"localPty:spawn" => {
					// `localPty:spawn` is Cocoon's Sky bridge path; preserve
					// the full `{id, name, pid}` shape because the older Wind
					// callers expect it. New `localPty:createProcess` and
					// `localPty:start` follow VS Code's typed contract below.
					dev_log!("terminal", "{}", command);
					handle_terminal_create(runtime.clone(), args).await
				},
				"localPty:createProcess" => {
					dev_log!("terminal", "{}", command);
					match handle_terminal_create(runtime.clone(), args).await {
						Ok(Response) => {
							// Extract the integer id - this is what
							// `IPtyService.createProcess` is contractually
							// required to return.
							let TerminalIdOption = Response.get("id").and_then(serde_json::Value::as_u64);
							match TerminalIdOption {
								Some(TerminalId) if TerminalId > 0 => Ok(serde_json::json!(TerminalId)),
								Some(_) | None => {
									// Defensive: if `CreateTerminal` returned
									// without a usable id (shape drift or
									// `GetNextTerminalIdentifier` regression),
									// surface the error to the workbench
									// instead of returning `0`. The workbench
									// would otherwise bind `LocalPty(0, …)`
									// and every subsequent `_proxy.input(0,
									// data)` would fail silently because no
									// PTY with id=0 exists - keystrokes get
									// swallowed with no diagnostic.
									dev_log!(
										"terminal",
										"error: [localPty:createProcess] CreateTerminal returned no usable id; \
										 response={:?}",
										Response
									);
									Err(format!(
										"localPty:createProcess: CreateTerminal returned no terminal id (response={})",
										Response
									))
								},
							}
						},
						Err(Error) => Err(Error),
					}
				},
				"localPty:start" => {
					// Eager-spawn pattern: `TerminalProvider::CreateTerminal`
					// already started the shell and reader task during
					// `localPty:createProcess`. `start` is a no-op that just
					// completes the workbench's launch promise. Returning
					// `Value::Null` matches `IPtyService.start`'s
					// `Promise<ITerminalLaunchError | ITerminalLaunchResult |
					// undefined>` (`undefined` branch). Routing this back
					// through `handle_terminal_create` would spawn a SECOND
					// PTY for the same workbench terminal - the user-visible
					// pane is bound to id=1 from `createProcess`, but a
					// shadow PTY (id=2) starts and streams data nobody
					// renders.
					dev_log!("terminal", "{} no-op (eager-spawn)", command);
					Ok(Value::Null)
				},
				"localPty:input" | "localPty:write" => {
					dev_log!("terminal", "{}", command);
					handle_terminal_send_text(runtime.clone(), args).await
				},
				"localPty:shutdown" | "localPty:dispose" => {
					dev_log!("terminal", "{}", command);
					handle_terminal_dispose(runtime.clone(), args).await
				},
				"localPty:resize" => {
					dev_log!("terminal", "localPty:resize");
					// Forward through the Terminal.Resize effect so the PTY master
					// receives SIGWINCH. Arguments from VS Code arrive as either
					// `[id, cols, rows]` or `{ id, cols, rows }`; accept both.
					//
					// Defensive clamping: portable-pty's `master.resize()`
					// crashes the IO thread with "size out of range" on
					// `cols=0` or `rows=0` (the workbench occasionally
					// emits 0×0 during pane drag-storms before the
					// `requestAnimationFrame` settle). Clamp to sane
					// minimums so a transient micro-size never tears
					// down the shell.
					let (TerminalId, Columns, Rows) = {
						let First = args.first().cloned().unwrap_or(Value::Null);
						if First.is_object() {
							let Id = First.get("id").and_then(|V| V.as_u64()).unwrap_or(0);
							let C = First.get("cols").and_then(|V| V.as_u64()).unwrap_or(80) as u16;
							let R = First.get("rows").and_then(|V| V.as_u64()).unwrap_or(24) as u16;
							(Id, C, R)
						} else {
							let Id = args.get(0).and_then(|V| V.as_u64()).unwrap_or(0);
							let C = args.get(1).and_then(|V| V.as_u64()).unwrap_or(80) as u16;
							let R = args.get(2).and_then(|V| V.as_u64()).unwrap_or(24) as u16;
							(Id, C, R)
						}
					};
					if TerminalId == 0 {
						Ok(Value::Null)
					} else {
						let Columns = if Columns == 0 { 1 } else { Columns };
						let Rows = if Rows == 0 { 1 } else { Rows };
						use CommonLibrary::{
							Environment::Requires::Requires,
							Terminal::TerminalProvider::TerminalProvider,
						};
						let Provider:Arc<dyn TerminalProvider> = runtime.Environment.Require();
						match Provider.ResizeTerminal(TerminalId, Columns, Rows).await {
							Ok(_) => Ok(Value::Null),
							Err(Error) => {
								// Resize on a disposed terminal is a common
								// race during shutdown - the workbench layout
								// pass fires after the user types `exit`, the
								// PTY closes, and the resize call lands on a
								// dropped master. Logging at warn instead of
								// error keeps the noise down. Returning
								// `Value::Null` (rather than a hard error)
								// lets the workbench's resize loop continue
								// instead of stalling on the failed promise.
								dev_log!(
									"terminal",
									"warn: localPty:resize id={} cols={} rows={} failed: {}",
									TerminalId,
									Columns,
									Rows,
									Error
								);
								Ok(Value::Null)
							},
						}
					}
				},
				"localPty:acknowledgeDataEvent" => {
					// xterm flow-control heartbeat; no-op on Mountain side.
					Ok(Value::Null)
				},
				// The remaining `localPty:*` endpoints declared by VS Code's
				// `ILocalPtyService` are lifecycle-/title-style hooks the extension
				// host calls even when there is no terminal running. They become
				// no-ops here so the workbench doesn't deadlock on a missing route.
				"localPty:processBinary"
				| "localPty:attachToProcess"
				| "localPty:detachFromProcess"
				| "localPty:orphanQuestionReply"
				| "localPty:updateTitle"
				| "localPty:updateIcon"
				| "localPty:refreshProperty"
				| "localPty:updateProperty"
				| "localPty:getRevivedPtyNewId"
				| "localPty:freePortKillProcess"
				| "localPty:reviveTerminalProcesses"
				| "localPty:getBackendOS"
				| "localPty:installAutoReply"
				| "localPty:uninstallAllAutoReplies"
				| "localPty:serializeTerminalState" => Ok(Value::Null),

				// =====================================================================
				// Update service
				// =====================================================================
				"update:_getInitialState" => {
					dev_log!("update", "update:_getInitialState");
					Ok(json!({ "type": "idle", "updateType": 0 }))
				},
				"update:isLatestVersion" => {
					dev_log!("update", "update:isLatestVersion");
					Ok(json!(true))
				},
				"update:checkForUpdates" => {
					dev_log!("update", "update:checkForUpdates");
					Ok(Value::Null)
				},
				"update:downloadUpdate" => {
					dev_log!("update", "update:downloadUpdate");
					Ok(Value::Null)
				},
				"update:applyUpdate" => {
					dev_log!("update", "update:applyUpdate");
					Ok(Value::Null)
				},
				"update:quitAndInstall" => {
					dev_log!("update", "update:quitAndInstall");
					Ok(Value::Null)
				},

				// =====================================================================
				// Menubar
				// =====================================================================
				//
				// VS Code emits `updateMenubar` every time a relevant state flips:
				// active editor, dirty marker, selection. A cold boot fires the call
				// ~20× in the first few seconds, and every one triggers an AppKit
				// re-render on macOS (≈ 200 ms each). We coalesce adjacent calls
				// through a 50 ms debouncer so only the last pending state actually
				// hits the native menu. Semantics match VS Code's
				// `ElectronMenubarControl._updateMenu` scheduler.
				"menubar:updateMenubar" => {
					use std::{
						sync::{Arc, Mutex as StandardMutex, OnceLock},
						time::Duration,
					};

					use tokio::task::JoinHandle;
					type MenubarCell = StandardMutex<(Option<JoinHandle<()>>, u64)>;
					static MENUBAR_DEBOUNCE:OnceLock<Arc<MenubarCell>> = OnceLock::new();
					let Cell = MENUBAR_DEBOUNCE.get_or_init(|| Arc::new(StandardMutex::new((None, 0)))).clone();

					if let Ok(mut Guard) = Cell.lock() {
						if let Some(Pending) = Guard.0.take() {
							Pending.abort();
						}
						Guard.1 = Guard.1.saturating_add(1);
						let CellForTask = Cell.clone();
						Guard.0 = Some(tokio::spawn(async move {
							tokio::time::sleep(Duration::from_millis(50)).await;
							let Coalesced = if let Ok(mut Post) = CellForTask.lock() {
								let N = Post.1;
								Post.1 = 0;
								Post.0 = None;
								N
							} else {
								0
							};
							dev_log!("menubar", "menubar:updateMenubar (applied, coalesced {} pending)", Coalesced);
						}));
					} else {
						dev_log!("menubar", "menubar:updateMenubar (debouncer lock poisoned)");
					}
					Ok(Value::Null)
				},

				// =====================================================================
				// URL handler
				// =====================================================================
				"url:registerExternalUriOpener" => {
					dev_log!("url", "url:registerExternalUriOpener");
					Ok(Value::Null)
				},

				// =====================================================================
				// Encryption
				// =====================================================================
				"encryption:encrypt" => {
					dev_log!("encryption", "encryption:encrypt");
					Ok(json!(""))
				},
				"encryption:decrypt" => {
					dev_log!("encryption", "encryption:decrypt");
					Ok(json!(""))
				},

				// =====================================================================
				// Extension host starter
				// =====================================================================
				"extensionHostStarter:createExtensionHost" => {
					dev_log!("exthost", "extensionHostStarter:createExtensionHost");
					Ok(json!({ "id": "1" }))
				},
				"extensionHostStarter:start" => {
					// The renderer uses this PID to correlate extension-host-side
					// debug adapters with the actual Node.js process. That process
					// is Cocoon, not Mountain - returning `std::process::id()`
					// here would point the debugger at Mountain's Rust binary.
					// Fall back to Mountain's PID only if Cocoon hasn't spawned
					// yet (should not happen for a real extension-host start).
					let Pid =
						crate::ProcessManagement::CocoonManagement::GetCocoonPid().unwrap_or_else(std::process::id);
					dev_log!("exthost", "extensionHostStarter:start pid={}", Pid);
					Ok(json!({ "pid": Pid }))
				},
				"extensionHostStarter:kill" => {
					dev_log!("exthost", "extensionHostStarter:kill");
					Ok(Value::Null)
				},
				"extensionHostStarter:getExitInfo" => {
					dev_log!("exthost", "extensionHostStarter:getExitInfo");
					Ok(json!({ "code": null, "signal": null }))
				},

				// =====================================================================
				// Extension host message relay (Wind → Mountain → Cocoon)
				// =====================================================================
				"cocoon:extensionHostMessage" => {
					let ByteCount = args
						.first()
						.map(|P| P.get("data").and_then(|D| D.as_array()).map(|A| A.len()).unwrap_or(0))
						.unwrap_or(0);
					dev_log!("exthost", "cocoon:extensionHostMessage bytes={}", ByteCount);

					// Forward binary message to Cocoon via gRPC GenericNotification.
					// Fire-and-forget - the extension host protocol is async.
					let Payload = args.first().cloned().unwrap_or(Value::Null);
					tokio::spawn(async move {
						if let Err(Error) = crate::Vine::Client::SendNotification(
							"cocoon-main".to_string(),
							"extensionHostMessage".to_string(),
							Payload,
						)
						.await
						{
							dev_log!("exthost", "cocoon:extensionHostMessage forward failed: {}", Error);
						}
					});
					Ok(Value::Null)
				},

				// =====================================================================
				// Extension host debug service
				// =====================================================================
				"extensionhostdebugservice:reload" => {
					dev_log!("exthost", "extensionhostdebugservice:reload");
					// Trigger a real Cocoon restart via the shutdown notification
					// followed by a fresh bootstrap. For the current sprint we emit
					// the request for Wind so it can tear down caches, the actual
					// spawn lives downstream.
					use tauri::Emitter;
					if let Err(Error) = app_handle.emit(SkyEvent::ExtHostDebugReload.AsStr(), json!({})) {
						dev_log!("exthost", "warn: extensionhostdebugservice:reload emit failed: {}", Error);
					}
					Ok(Value::Null)
				},
				"extensionhostdebugservice:close" => {
					dev_log!("exthost", "extensionhostdebugservice:close");
					use tauri::Emitter;
					if let Err(Error) = app_handle.emit("sky://exthost/debug-close", json!({})) {
						dev_log!("exthost", "warn: extensionhostdebugservice:close emit failed: {}", Error);
					}
					Ok(Value::Null)
				},
				"extensionhostdebugservice:attachSession" | "extensionhostdebugservice:terminateSession" => {
					dev_log!("exthost", "{}", command);
					Ok(Value::Null)
				},

				// =====================================================================
				// Workspaces - additional commands
				// =====================================================================
				"workspaces:getRecentlyOpened" => {
					dev_log!("workspaces", "workspaces:getRecentlyOpened");
					ReadRecentlyOpened()
				},
				"workspaces:removeRecentlyOpened" => {
					dev_log!("workspaces", "workspaces:removeRecentlyOpened");
					let Uri = args.first().and_then(|V| V.as_str()).unwrap_or("").to_string();
					if !Uri.is_empty() {
						MutateRecentlyOpened(|List| {
							if let Some(Workspaces) = List.get_mut("workspaces").and_then(|V| V.as_array_mut()) {
								Workspaces
									.retain(|Entry| Entry.get("uri").and_then(|V| V.as_str()).unwrap_or("") != Uri);
							}
							if let Some(Files) = List.get_mut("files").and_then(|V| V.as_array_mut()) {
								Files.retain(|Entry| Entry.get("uri").and_then(|V| V.as_str()).unwrap_or("") != Uri);
							}
						});
					}
					Ok(Value::Null)
				},
				"workspaces:addRecentlyOpened" => {
					dev_log!("workspaces", "workspaces:addRecentlyOpened");
					// VS Code passes `[{ workspace?, folderUri?, fileUri?, label? }, …]`.
					let Entries:Vec<Value> = args.first().and_then(|V| V.as_array()).cloned().unwrap_or_default();
					if !Entries.is_empty() {
						MutateRecentlyOpened(|List| {
							let Workspaces = List
								.get_mut("workspaces")
								.and_then(|V| V.as_array_mut())
								.map(|V| std::mem::take(V))
								.unwrap_or_default();
							let Files = List
								.get_mut("files")
								.and_then(|V| V.as_array_mut())
								.map(|V| std::mem::take(V))
								.unwrap_or_default();
							let mut MergedWorkspaces = Workspaces;
							let mut MergedFiles = Files;
							for Entry in Entries {
								let Folder = Entry
									.get("folderUri")
									.cloned()
									.or_else(|| Entry.get("workspace").and_then(|W| W.get("configPath").cloned()));
								let File = Entry.get("fileUri").cloned();
								if let Some(FolderUri) = Folder.and_then(|V| v_str(&V)) {
									MergedWorkspaces
										.retain(|E| E.get("uri").and_then(|V| V.as_str()).unwrap_or("") != FolderUri);
									let mut Item = serde_json::Map::new();
									Item.insert("uri".into(), json!(FolderUri));
									if let Some(Label) = Entry.get("label").and_then(|V| V.as_str()) {
										Item.insert("label".into(), json!(Label));
									}
									MergedWorkspaces.insert(0, Value::Object(Item));
								}
								if let Some(FileUri) = File.and_then(|V| v_str(&V)) {
									MergedFiles
										.retain(|E| E.get("uri").and_then(|V| V.as_str()).unwrap_or("") != FileUri);
									let mut Item = serde_json::Map::new();
									Item.insert("uri".into(), json!(FileUri));
									MergedFiles.insert(0, Value::Object(Item));
								}
							}
							// Cap at 50 each - matches VS Code's default in
							// `src/vs/platform/workspaces/common/workspaces.ts`.
							MergedWorkspaces.truncate(50);
							MergedFiles.truncate(50);
							List.insert("workspaces".into(), Value::Array(MergedWorkspaces));
							List.insert("files".into(), Value::Array(MergedFiles));
						});
					}
					Ok(Value::Null)
				},
				"workspaces:clearRecentlyOpened" => {
					dev_log!("workspaces", "workspaces:clearRecentlyOpened");
					MutateRecentlyOpened(|List| {
						List.insert("workspaces".into(), json!([]));
						List.insert("files".into(), json!([]));
					});
					Ok(Value::Null)
				},
				"workspaces:enterWorkspace" => {
					dev_log!("workspaces", "workspaces:enterWorkspace");
					Ok(Value::Null)
				},
				"workspaces:createUntitledWorkspace" => {
					dev_log!("workspaces", "workspaces:createUntitledWorkspace");
					Ok(Value::Null)
				},
				"workspaces:deleteUntitledWorkspace" => {
					dev_log!("workspaces", "workspaces:deleteUntitledWorkspace");
					Ok(Value::Null)
				},
				"workspaces:getWorkspaceIdentifier" => {
					// Return a stable identifier derived from the first workspace
					// folder's URI so VS Code's caching (recently-opened, per-workspace
					// storage, window-title derivation) keys off the real workspace
					// rather than the "untitled" fallback. `{ id, configPath }` is
					// VS Code's expected shape for a multi-root workspace identifier;
					// we only use single-root so configPath stays null.
					let Workspace = &runtime.Environment.ApplicationState.Workspace;
					let Folders = Workspace.GetWorkspaceFolders();
					if let Some(First) = Folders.first() {
						use std::{
							collections::hash_map::DefaultHasher,
							hash::{Hash, Hasher},
						};
						let mut Hasher = DefaultHasher::new();
						First.URI.as_str().hash(&mut Hasher);
						let Id = format!("{:016x}", Hasher.finish());
						Ok(json!({
							"id": Id,
							"configPath": Value::Null,
							"uri": First.URI.to_string(),
						}))
					} else {
						Ok(Value::Null)
					}
				},
				"workspaces:getDirtyWorkspaces" => Ok(json!([])),

				// Git (localGit channel) - implements stock VS Code's
				// ILocalGitService surface plus `exec` / `isAvailable` for
				// the built-in Git extension. Handlers spawn native `git`
				// via tokio::process. See Batch 4 in HANDOFF §-10.
				"git:exec" => {
					dev_log!("git", "git:exec");
					Git::HandleExec(args).await
				},
				"git:clone" => {
					dev_log!("git", "git:clone");
					Git::HandleClone(args).await
				},
				"git:pull" => {
					dev_log!("git", "git:pull");
					Git::HandlePull(args).await
				},
				"git:checkout" => {
					dev_log!("git", "git:checkout");
					Git::HandleCheckout(args).await
				},
				"git:revParse" => {
					dev_log!("git", "git:revParse");
					Git::HandleRevParse(args).await
				},
				"git:fetch" => {
					dev_log!("git", "git:fetch");
					Git::HandleFetch(args).await
				},
				"git:revListCount" => {
					dev_log!("git", "git:revListCount");
					Git::HandleRevListCount(args).await
				},
				"git:cancel" => {
					dev_log!("git", "git:cancel");
					Git::HandleCancel(args).await
				},
				"git:isAvailable" => {
					dev_log!("git", "git:isAvailable");
					Git::HandleIsAvailable(args).await
				},

				// Tree-view child lookup from the renderer side. Mirrors the
				// Cocoon→Mountain `GetTreeChildren` gRPC path (see
				// `RPC/CocoonService/TreeView.rs::GetTreeChildren`) but is
				// invoked by the Wind/Sky tree-view bridge so the UI can
				// request children directly without waiting for Cocoon to
				// ask first. Payload: `[{ viewId, treeItemHandle? }]`.
				"tree:getChildren" => {
					let ViewId = args
						.first()
						.and_then(|V| V.get("viewId").or_else(|| V.get(0)))
						.and_then(Value::as_str)
						.unwrap_or("")
						.to_string();
					let ItemHandle = args
						.first()
						.and_then(|V| V.get("treeItemHandle").or_else(|| V.get(1)))
						.and_then(Value::as_str)
						.unwrap_or("")
						.to_string();
					dev_log!(
						"tree-view",
						"[TreeView] invoke:getChildren view={} parent={}",
						ViewId,
						ItemHandle
					);
					if ViewId.is_empty() {
						Err("tree:getChildren requires viewId".to_string())
					} else {
						let Parameters = json!({
							"viewId": ViewId,
							"treeItemHandle": ItemHandle,
						});
						// Boot-race: the workbench's Explorer view fires
						// `tree:getChildren` ~700 log lines before
						// Cocoon's gRPC client finishes handshaking.
						// Without this wait the first call returns
						// `ClientNotConnected`, the workbench caches an
						// empty list, and the user sees an empty
						// Explorer until they manually refresh. Wait up
						// to 1500 ms for the connection to land before
						// dispatching - this no-ops once Cocoon is
						// connected (the typical case), so it only
						// costs us latency on the very first call.
						let _ = crate::Vine::Client::WaitForClientConnection("cocoon-main", 1500).await;
						// Tree-view RPCs are user-interactive: a 5 second
						// wait shows the user a spinner and silently fails
						// the extension's Promise on timeout. 1500 ms is
						// a reasonable upper bound for "extension is
						// healthy and producing children" on this hardware
						// class - real workloads (gitlens fileHistory,
						// rust-analyzer typeHierarchy) finish in <500 ms.
						// Slow producers fall through to the empty-array
						// path and the workbench schedules its own retry
						// when the view scrolls back into view.
						match crate::Vine::Client::SendRequest(
							"cocoon-main",
							"$provideTreeChildren".to_string(),
							Parameters,
							1500,
						)
						.await
						{
							// Defensive shape check: the workbench's
							// tree-view consumer expects either an
							// `items` array or a top-level array.
							// `null` / `undefined` from a misbehaving
							// extension would be passed through and
							// trigger `TypeError: Cannot read property
							// 'length' of null` in the renderer. Force
							// to `{items: []}` for any non-conforming
							// shape so the renderer always has
							// iterable data.
							Ok(Value_) => {
								match &Value_ {
									Value::Object(_) | Value::Array(_) => Ok(Value_),
									_ => Ok(json!({ "items": [] })),
								}
							},
							Err(Error) => {
								// Common case: an extension's tree
								// data-provider rejects (npm extension
								// crashes on a malformed
								// `package.json`, gitlens hits an
								// expired pull-request fetch, etc.).
								// First failure per view is logged so
								// developers can see the cause; later
								// failures of the same view-id are
								// silenced via the file-sink-only
								// path so the dev log doesn't fill
								// with hundreds of identical lines
								// while the user is browsing tree
								// nodes that all hit the same
								// extension bug.
								crate::IPC::DevLog::DebugOnce(
									"tree-view",
									&format!("get-children-error:{}", ViewId),
									&format!(
										"[TreeView] invoke:getChildren error view={} err={:?} (further occurrences \
										 silenced)",
										ViewId, Error
									),
								);
								Ok(json!({ "items": [] }))
							},
						}
					}
				},

				// SkyBridge calls this after installing every `sky://*` Tauri
				// listener. Mountain → Sky `app.emit()` events are NOT
				// buffered: any emit fired before the listener was installed
				// is silently dropped. In the bundled-electron profile,
				// extension activation (which triggers
				// `register_scm_provider` and `$tree:register` notifications
				// through Cocoon) starts ~580 log lines before the Sky
				// bundle finishes booting (~1995 lines). Without this
				// replay, all tree-view + SCM register events are lost and
				// the Activity Bar / sidebar comes up empty even though
				// state-side everything registered correctly.
				"sky:replay-events" => {
					use tauri::Emitter;
					let mut TreeViewCount:usize = 0;
					let mut ScmCount:usize = 0;
					let mut CommandCount:usize = 0;
					let mut TerminalCount:usize = 0;
					let mut TerminalDataBytes:usize = 0;
					if let Ok(TreeViews) = runtime.Environment.ApplicationState.Feature.TreeViews.ActiveTreeViews.lock()
					{
						for (ViewId, Dto) in TreeViews.iter() {
							let Payload = serde_json::json!({
								"viewId": ViewId,
								"options": {
									"canSelectMany": Dto.CanSelectMany,
									"showCollapseAll": Dto.HasHandleDrag,
									"title": Dto.Title.clone().unwrap_or_default(),
								},
							});
							if app_handle.emit("sky://tree-view/create", Payload).is_ok() {
								TreeViewCount += 1;
							}
						}
					}
					// SCM replay uses the stored `Identifier` field on the
					// provider DTO ("git", "github", "hg", …) so any SCM
					// provider Cocoon registers replays with its original
					// id - not just the built-in `vscode.git` extension.
					// Pre-DTO-Identifier-field DTOs default `Identifier` to
					// "" (serde default); fall back to "git" in that case
					// because the only SCM provider in production today is
					// `vscode.git` and a stale state file with empty id is
					// the realistic upgrade-path mismatch.
					if let Ok(ScmProviders) = runtime
						.Environment
						.ApplicationState
						.Feature
						.Markers
						.SourceControlManagementProviders
						.lock()
					{
						for (Handle, Dto) in ScmProviders.iter() {
							let RootUriStr = Dto
								.RootURI
								.as_ref()
								.and_then(|V| V.get("external").or_else(|| V.get("path")))
								.and_then(serde_json::Value::as_str)
								.unwrap_or("")
								.to_string();
							let ScmId = if Dto.Identifier.is_empty() {
								"git".to_string()
							} else {
								Dto.Identifier.clone()
							};
							let Payload = serde_json::json!({
								"scmId": ScmId,
								"label": Dto.Label,
								"rootUri": RootUriStr,
								"extensionId": "",
								"handle": *Handle,
							});
							if app_handle.emit("sky://scm/register", Payload).is_ok() {
								ScmCount += 1;
							}
						}
					}
					// Replay extension-registered commands so the workbench's
					// `ICommandService` registry knows about them post-bridge-
					// install. Every `vscode.commands.registerCommand(...)`
					// in an extension fires `sky://command/register` from
					// `Vine/Server/Notification/RegisterCommand.rs`. Native
					// commands (Mountain's own Rust handlers) don't need
					// replay - they're not exposed via this channel.
					//
					// Emit ONE batched event with the whole array. Per-
					// command emits (one per registered command, ~1000+
					// during extension boot) saturated Tauri's shared
					// WKWebView IPC channel and starved keystroke
					// delivery. SkyBridge's `sky://command/register`
					// listener accepts either `{ id, commandId, kind }`
					// or `{ commands: [...] }` (see SkyBridge.ts).
					if let Ok(Commands) = runtime.Environment.ApplicationState.Extension.Registry.CommandRegistry.lock()
					{
						let mut Batch: Vec<serde_json::Value> = Vec::new();
						for (CommandId, Handler) in Commands.iter() {
							use crate::Environment::CommandProvider::CommandHandler;
							let Kind = match Handler {
								CommandHandler::Native(_) => continue,
								CommandHandler::Proxied { .. } => "extension",
							};
							Batch.push(serde_json::json!({
								"id": CommandId,
								"commandId": CommandId,
								"kind": Kind,
							}));
						}
						if !Batch.is_empty() {
							let Count = Batch.len();
							if app_handle
								.emit("sky://command/register", serde_json::json!({ "commands": Batch }))
								.is_ok()
							{
								CommandCount = Count;
							}
						}
					}
					// Replay terminals: each active terminal needs its `create`
					// event AND any buffered stdout the PTY reader produced
					// before SkyBridge's `listen("sky://terminal/*")` was
					// installed. Without this, the shell's first prompt
					// (zsh's MOTD, fish greeting, `direnv export`, …) is
					// silently dropped and the user sees an empty pane until
					// they type.
					if let Ok(Terminals) = runtime.Environment.ApplicationState.Feature.Terminals.ActiveTerminals.lock()
					{
						for (TerminalId, Arc) in Terminals.iter() {
							let (Name, Pid) = if let Ok(State) = Arc.lock() {
								(State.Name.clone(), State.OSProcessIdentifier.unwrap_or(0))
							} else {
								(String::new(), 0)
							};
							let CreatePayload = serde_json::json!({
								"id": *TerminalId,
								"name": Name,
								"pid": Pid,
							});
							if app_handle.emit("sky://terminal/create", CreatePayload).is_ok() {
								TerminalCount += 1;
							}
						}
					}
					for (TerminalId, Bytes) in crate::Environment::TerminalProvider::DrainTerminalOutputBuffer() {
						let DataString = String::from_utf8_lossy(&Bytes).to_string();
						TerminalDataBytes += Bytes.len();
						let _ = app_handle.emit(
							"sky://terminal/data",
							serde_json::json!({ "id": TerminalId, "data": DataString }),
						);
					}
					dev_log!(
						"sky-emit",
						"[SkyEmit] replay-events tree-views={} scm={} commands={} terminals={} terminal-bytes={}",
						TreeViewCount,
						ScmCount,
						CommandCount,
						TerminalCount,
						TerminalDataBytes
					);
					Ok(serde_json::json!({
						"treeViews": TreeViewCount,
						"scmProviders": ScmCount,
						"commands": CommandCount,
						"terminals": TerminalCount,
						"terminalDataBytes": TerminalDataBytes,
					}))
				},

				// Atom L2: unknown-command fallback consults the Channel registry so
				// the log distinguishes three states:
				//   1. typo / never-registered wire string (registry::from_str Err)
				//   2. registered but dispatch missing (registry OK but arm absent)
				//   3. legitimately unknown
				// Case (2) is the shape of the VSIX stub bug before K2 landed - an
				// entry present in the registry with no handler. Making it visible
				// turns silent drift into a loud dev-log line.
				_ => {
					use std::str::FromStr;
					match CommonLibrary::IPC::Channel::Channel::from_str(&command) {
						Ok(KnownChannel) => {
							dev_log!(
								"ipc",
								"error: [WindServiceHandlers] Channel {:?} is registered but has no dispatch arm",
								KnownChannel
							);
							Err(format!("IPC channel registered but unimplemented: {}", command))
						},
						Err(_) => {
							dev_log!("ipc", "error: [WindServiceHandlers] Unknown IPC command: {}", command);
							Err(format!("Unknown IPC command: {}", command))
						},
					}
				},
			};

			if ResultSender.send(MatchResult).is_err() {
				dev_log!(
					"ipc",
					"warn: [WindServiceHandlers] IPC result receiver dropped before dispatch completed"
				);
			}
		},
		CommandPriority,
	);

	let Result = match ResultReceiver.await {
		Ok(Dispatched) => Dispatched,
		Err(_) => {
			dev_log!(
				"ipc",
				"error: [WindServiceHandlers] IPC task cancelled before producing a result"
			);
			Err("IPC task cancelled before result was produced".to_string())
		},
	};

	// Emit OTLP span for every IPC call - visible in Jaeger at localhost:16686
	let IsErr = Result.is_err();
	let SpanName = if IsErr {
		format!("ipc:{}:error", command)
	} else {
		format!("ipc:{}", command)
	};
	crate::otel_span!(&SpanName, OTLPStart, &[("ipc.command", command.as_str())]);

	// Atom I13: paired entry/exit line per invoke. `invoke: <cmd>` on the way
	// in (emitted at the top of this fn); `done: <cmd> ok=… t_ns=…` on the
	// way out. A `grep "logger:log"` before showed only the entry half;
	// having both halves makes latency diagnosis a single pipe:
	//     grep "logger:log" Mountain.dev.log | awk '…'
	// without hopping across Jaeger. High-frequency commands still skip the
	// entry line but DO emit an exit - frequencies still aggregate, but each
	// is individually accounted for.
	if !IsHighFrequencyCommand {
		let ElapsedNanos = crate::IPC::DevLog::NowNano().saturating_sub(OTLPStart);
		dev_log!("ipc", "done: {} ok={} t_ns={}", command, !IsErr, ElapsedNanos);
	}

	Result
}

pub fn register_wind_ipc_handlers(app_handle:&tauri::AppHandle) -> Result<(), String> {
	dev_log!("lifecycle", "registering IPC handlers");

	// Note: These handlers are automatically registered when included in the
	// Tauri invoke_handler macro in the main binary

	Ok(())
}
