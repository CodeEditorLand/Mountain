//! Wind Service Handlers - dispatcher and sub-module aggregator.
//! Domain files handle the individual handler implementations.

pub mod Cocoon;

#[path = "Commands/mod.rs"]
pub mod Commands;

#[path = "Configuration/mod.rs"]
pub mod Configuration;

pub mod Encryption;

pub mod Extension;

pub mod ExtensionHost;

pub mod Extensions;

pub mod FileSystem;

pub mod Git;

pub mod Model;

pub mod NativeDialog;

pub mod NativeHost;

pub mod Navigation;

pub mod Output;

#[path = "Search/mod.rs"]
pub mod Search;

pub mod Sky;

pub mod Storage;

pub mod Terminal;

pub mod UI;

pub mod TreeView;

pub mod Update;

pub mod Utilities;

// Local `use X::*;` (NOT `pub use`): brings the domain handler names into
// this file's scope so the dispatch match arms below can call
// `handle_foo(...)` unqualified. Local `use` is scoped to this file only;
// external callers must spell the full path
// (`WindServiceHandlers::Utilities::foo`).
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use Cocoon::{
	ExtensionHostMessage::Fn as CocoonExtensionHostMessage,
	Notify::Fn as CocoonNotify,
	Request::Fn as CocoonRequest,
};
use ExtensionHost::{
	DebugServiceClose::Fn as ExtensionHostDebugClose,
	DebugServiceReload::Fn as ExtensionHostDebugReload,
	StarterCreate::Fn as ExtensionHostStarterCreate,
	StarterGetExitInfo::Fn as ExtensionHostStarterGetExitInfo,
	StarterKill::Fn as ExtensionHostStarterKill,
	StarterStart::Fn as ExtensionHostStarterStart,
	StarterWaitForExit::Fn as ExtensionHostStarterWaitForExit,
};
use Sky::ReplayEvents::Fn as SkyReplayEvents;
use TreeView::GetChildren::Fn as TreeGetChildren;
use Update::{
	ApplyUpdate::Fn as UpdateApplyUpdate,
	CheckForUpdates::Fn as UpdateCheckForUpdates,
	DownloadUpdate::Fn as UpdateDownloadUpdate,
	GetInitialState::Fn as UpdateGetInitialState,
	IsLatestVersion::Fn as UpdateIsLatestVersion,
	QuitAndInstall::Fn as UpdateQuitAndInstall,
};
use Commands::{Execute::Fn as CommandsExecute, GetAll::Fn as CommandsGetAll};
use Configuration::{
	EnvironmentGet::Fn as EnvironmentGet,
	Get::Fn as ConfigurationGet,
	Update::Fn as ConfigurationUpdate,
	Workbench::Fn as WorkbenchConfiguration,
};
use Encryption::{Decrypt::Fn as Decrypt, Encrypt::Fn as Encrypt};
use Extensions::{
	ExtensionsGet::Fn as ExtensionsGet,
	ExtensionsGetAll::Fn as ExtensionsGetAll,
	ExtensionsGetInstalled::Fn as ExtensionsGetInstalled,
	ExtensionsIsActive::Fn as ExtensionsIsActive,
};
use FileSystem::{
	Managed::{
		FileCopy::Fn as FileCopy,
		FileDelete::Fn as FileDelete,
		FileExists::Fn as FileExists,
		FileMkdir::Fn as FileMkdir,
		FileMove::Fn as FileMove,
		FileRead::Fn as FileRead,
		FileReadBinary::Fn as FileReadBinary,
		FileReaddir::Fn as FileReaddir,
		FileStat::Fn as FileStat,
		FileWrite::Fn as FileWrite,
		FileWriteBinary::Fn as FileWriteBinary,
	},
	Native::{
		FileCloneNative::Fn as FileCloneNative,
		FileCloseFd::Fn as FileCloseFd,
		FileDeleteNative::Fn as FileDeleteNative,
		FileExistsNative::Fn as FileExistsNative,
		FileMkdirNative::Fn as FileMkdirNative,
		FileOpenFd::Fn as FileOpenFd,
		FileReadNative::Fn as FileReadNative,
		FileReaddirNative::Fn as FileReaddirNative,
		FileRealpath::Fn as FileRealpath,
		FileRenameNative::Fn as FileRenameNative,
		FileStatNative::Fn as FileStatNative,
		FileUnwatch::Fn as FileUnwatch,
		FileWatch::Fn as FileWatch,
		FileWriteNative::Fn as FileWriteNative,
	},
};
use Model::{
	ModelClose::Fn as ModelClose,
	ModelGet::Fn as ModelGet,
	ModelGetAll::Fn as ModelGetAll,
	ModelOpen::Fn as ModelOpen,
	ModelUpdateContent::Fn as ModelUpdateContent,
	TextfileRead::Fn as TextfileRead,
	TextfileSave::Fn as TextfileSave,
	TextfileWrite::Fn as TextfileWrite,
};
use NativeHost::{
	ClipboardHas::Fn as NativeHasClipboard,
	ClipboardReadBuffer::Fn as NativeReadClipboardBuffer,
	ClipboardReadFindText::Fn as NativeReadClipboardFindText,
	ClipboardReadImage::Fn as NativeReadImage,
	ClipboardReadText::Fn as NativeReadClipboardText,
	ClipboardTriggerPaste::Fn as NativeTriggerPaste,
	ClipboardWriteBuffer::Fn as NativeWriteClipboardBuffer,
	ClipboardWriteFindText::Fn as NativeWriteClipboardFindText,
	ClipboardWriteText::Fn as NativeWriteClipboardText,
	Exit::Fn as Exit,
	FindFreePort::Fn as NativeFindFreePort,
	GetColorScheme::Fn as NativeGetColorScheme,
	GetEnvironmentPaths::Fn as NativeGetEnvironmentPaths,
	InstallShellCommand::Fn as InstallShellCommand,
	IsFullscreen::Fn as NativeIsFullscreen,
	IsMaximized::Fn as NativeIsMaximized,
	IsRunningUnderARM64Translation::Fn as NativeIsRunningUnderARM64Translation,
	KillProcess::Fn as KillProcess,
	MoveItemToTrash::Fn as NativeMoveItemToTrash,
	OSProperties::Fn as NativeOSProperties,
	OSStatistics::Fn as NativeOSStatistics,
	OpenDevTools::Fn as OpenDevTools,
	OpenExternal::Fn as OpenExternal,
	PickFolder::Fn as NativePickFolder,
	Quit::Fn as Quit,
	Relaunch::Fn as Relaunch,
	Reload::Fn as Reload,
	ShowItemInFolder::Fn as ShowItemInFolder,
	ShowMessageBox::Fn as NativeShowMessageBox,
	ShowOpenDialog::Fn as NativeShowOpenDialog,
	ShowSaveDialog::Fn as NativeShowSaveDialog,
	ShowSaveDialogUI::Fn as UserInterfaceShowSaveDialog,
	ToggleDevTools::Fn as ToggleDevTools,
	UninstallShellCommand::Fn as UninstallShellCommand,
};
use Navigation::{
	HistoryCanGoBack::Fn as HistoryCanGoBack,
	HistoryCanGoForward::Fn as HistoryCanGoForward,
	HistoryClear::Fn as HistoryClear,
	HistoryGetStack::Fn as HistoryGetStack,
	HistoryGoBack::Fn as HistoryGoBack,
	HistoryGoForward::Fn as HistoryGoForward,
	HistoryPush::Fn as HistoryPush,
	LabelGetBase::Fn as LabelGetBase,
	LabelGetURI::Fn as LabelGetURI,
	LabelGetWorkspace::Fn as LabelGetWorkspace,
};
use Output::{
	OutputAppend::Fn as OutputAppend,
	OutputAppendLine::Fn as OutputAppendLine,
	OutputClear::Fn as OutputClear,
	OutputCreate::Fn as OutputCreate,
	OutputShow::Fn as OutputShow,
};
use Search::{FindFiles::Fn as SearchFindFiles, FindInFiles::Fn as SearchFindInFiles};
use Storage::{
	StorageDelete::Fn as StorageDelete,
	StorageGet::Fn as StorageGet,
	StorageGetItems::Fn as StorageGetItems,
	StorageKeys::Fn as StorageKeys,
	StorageSet::Fn as StorageSet,
	StorageUpdateItems::Fn as StorageUpdateItems,
};
use Terminal::{
	AttachToProcess::Fn as AttachToProcess,
	DetachFromProcess::Fn as DetachFromProcess,
	LocalPTYCreateProcess::Fn as LocalPTYCreateProcess,
	LocalPTYFreePortKillProcess::Fn as LocalPTYFreePortKillProcess,
	LocalPTYGetDefaultShell::Fn as LocalPTYGetDefaultShell,
	LocalPTYGetEnvironment::Fn as LocalPTYGetEnvironment,
	LocalPTYGetProfiles::Fn as LocalPTYGetProfiles,
	LocalPTYResize::Fn as LocalPTYResize,
	ReviveTerminalProcesses::Fn as ReviveTerminalProcesses,
	SerializeTerminalState::Fn as SerializeTerminalState,
	TerminalCreate::Fn as TerminalCreate,
	TerminalDispose::Fn as TerminalDispose,
	TerminalHide::Fn as TerminalHide,
	TerminalSendText::Fn as TerminalSendText,
	TerminalShow::Fn as TerminalShow,
};
use UI::{
	DecorationsClear::Fn as DecorationsClear,
	DecorationsGet::Fn as DecorationsGet,
	DecorationsGetMany::Fn as DecorationsGetMany,
	DecorationsSet::Fn as DecorationsSet,
	KeybindingAdd::Fn as KeybindingAdd,
	KeybindingGetAll::Fn as KeybindingGetAll,
	KeybindingLookup::Fn as KeybindingLookup,
	KeybindingRemove::Fn as KeybindingRemove,
	LifecycleGetPhase::Fn as LifecycleGetPhase,
	LifecycleRequestShutdown::Fn as LifecycleRequestShutdown,
	LifecycleWhenPhase::Fn as LifecycleWhenPhase,
	NotificationEndProgress::Fn as NotificationEndProgress,
	NotificationShow::Fn as NotificationShow,
	NotificationShowProgress::Fn as NotificationShowProgress,
	NotificationUpdateProgress::Fn as NotificationUpdateProgress,
	ProgressBegin::Fn as ProgressBegin,
	ProgressEnd::Fn as ProgressEnd,
	ProgressReport::Fn as ProgressReport,
	QuickInputShowInputBox::Fn as QuickInputShowInputBox,
	QuickInputShowQuickPick::Fn as QuickInputShowQuickPick,
	ThemesGetActive::Fn as ThemesGetActive,
	ThemesList::Fn as ThemesList,
	ThemesSet::Fn as ThemesSet,
	WorkingCopyGetAllDirty::Fn as WorkingCopyGetAllDirty,
	WorkingCopyGetDirtyCount::Fn as WorkingCopyGetDirtyCount,
	WorkingCopyIsDirty::Fn as WorkingCopyIsDirty,
	WorkingCopySetDirty::Fn as WorkingCopySetDirty,
	WorkspacesAddFolder::Fn as WorkspacesAddFolder,
	WorkspacesGetFolders::Fn as WorkspacesGetFolders,
	WorkspacesGetName::Fn as WorkspacesGetName,
	WorkspacesRemoveFolder::Fn as WorkspacesRemoveFolder,
};
use Utilities::{
	ApplicationRoot::{Get::Fn as get_static_application_root, Set::Fn as set_static_application_root},
	ChannelPriority::Fn as ResolveChannelPriority,
	FiddeeRoot::Fn as FiddeeRoot,
	JsonValueHelpers::{
		Fn as v_str,
		arg_bool,
		arg_bool_true,
		arg_f64,
		arg_i64,
		arg_str,
		arg_string,
		arg_string_or,
		arg_u64,
		arg_u64_or,
		arg_val,
		req_string,
	},
	MetadataEncoding::Fn as metadata_to_istat,
	PathExtraction::{Fn as extract_path_from_arg, percent_decode},
	RecentlyOpened::{
		Mutate::Fn as MutateRecentlyOpened,
		Path::Fn as RecentlyOpenedPath,
		Read::Fn as ReadRecentlyOpened,
	},
	UserdataDir::{
		Ensure::Fn as ensure_userdata_dirs,
		Get::Fn as get_userdata_base_dir,
		Set::Fn as set_userdata_base_dir,
	},
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
	LanguageFeature::{
		DTO::PositionDTO::PositionDTO,
		LanguageFeatureProviderRegistry::LanguageFeatureProviderRegistry,
	},
	Storage::StorageProvider::StorageProvider,
};

use crate::{
	ApplicationState::{
		DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
		State::{
			ApplicationState::ApplicationState,
			WorkspaceState::WorkspaceDelta::UpdateWorkspaceFoldersAndBroadcast,
		},
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

fn cocoon_payload(args:Vec<Value>) -> Value {
	match args.len() {
		0 => Value::Null,
		1 => args.into_iter().next().unwrap(),
		_ => Value::Array(args),
	}
}

macro_rules! forward_to_cocoon {
	($tag:literal, $command:ident, $Arguments:ident) => {{
		dev_log!("ipc", "{}: {} (→ Cocoon)", $tag, $command);
		let Payload = cocoon_payload($Arguments);
		let _ = crate::Vine::Client::WaitForClientConnection::Fn("cocoon-main", 3000).await;
		Ok(
			crate::Vine::Client::SendRequest::Fn("cocoon-main", $command.clone(), Payload, 10_000)
				.await
				.unwrap_or(Value::Null),
		)
	}};
}

/// Delegate an IPC arm to an atomic handler with a `dev_log!` prefix.
///
/// `rt` / `app` selects `RunTime.clone()` vs `ApplicationHandle.clone()`.
/// Third token: a `$tag:literal` path for the log category.
/// Optional `$msg:literal` after the tag: static message (else `"{}",
/// command`). Optional trailing `$Arguments:ident`: pass arguments to the
/// handler.
macro_rules! call {
	(rt, $tag:literal, $Fn:path, $Arguments:ident) => {{
		dev_log!($tag, "{}", command);
		$Fn(RunTime.clone(), $Arguments).await
	}};
	(rt, $tag:literal, $Fn:path) => {{
		dev_log!($tag, "{}", command);
		$Fn(RunTime.clone()).await
	}};
	(rt, $tag:literal, $msg:literal, $Fn:path, $Arguments:ident) => {{
		dev_log!($tag, $msg);
		$Fn(RunTime.clone(), $Arguments).await
	}};
	(rt, $tag:literal, $msg:literal, $Fn:path) => {{
		dev_log!($tag, $msg);
		$Fn(RunTime.clone()).await
	}};
	(app, $tag:literal, $Fn:path, $Arguments:ident) => {{
		dev_log!($tag, "{}", command);
		$Fn(ApplicationHandle.clone(), $Arguments).await
	}};
	(app, $tag:literal, $msg:literal, $Fn:path, $Arguments:ident) => {{
		dev_log!($tag, $msg);
		$Fn(ApplicationHandle.clone(), $Arguments).await
	}};
}

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
/// The local parameter names (`command` / `Arguments`) are preserved for diff
/// minimality; the frontend-facing contract (`method` / `params`) lives
/// entirely in `InvokeCommand.rs`.
pub async fn mountain_ipc_invoke(
	ApplicationHandle:AppHandle,

	command:String,

	Arguments:Vec<Value>,
) -> Result<Value, String> {
	// Determine high-frequency status first - used to skip OTLP timing,
	// dev-logs, span emission, and PostHog capture for noisy calls.
	let IsHighFrequencyCommand = matches!(
		command.as_str(),
		"logger:log"
			| "logger:info"
			| "logger:debug"
			| "logger:trace"
			| "logger:warn"
			| "logger:error"
			| "logger:critical"
			| "logger:flush"
			| "logger:setLevel"
			| "logger:getLevel"
			| "logger:registerLogger"
			| "logger:createLogger"
			| "logger:deregisterLogger"
			| "logger:getRegisteredLoggers"
			| "logger:setVisibility"
			| "log:registerLogger"
			| "log:createLogger"
			// File system - high-frequency VS Code workbench calls
			| "file:stat"
			| "file:readFile"
			| "file:readdir"
			| "file:writeFile"
			| "file:delete"
			| "file:rename"
			| "file:realpath"
			| "file:read"
			| "file:write"
			// fd-table ops - called per-file during project open cascades
			| "file:open"
			| "file:close"
			// Auto-save intent - fires once/second per dirty file
			| "textFile:save"
			// Storage - polled constantly by VS Code services
			| "storage:getItems"
			| "storage:updateItems"
			// Configuration - scoped-lookup hot path
			| "configuration:lookup"
			| "configuration:inspect"
			// Themes - queried on every decoration/token change
			| "themes:getColorTheme"
			// Output/Progress - emitted in tight loops
			| "output:append"
			| "progress:report"
			// Menubar - updated on every editor/selection change
			| "menubar:updateMenubar"
			// Ack-only event stubs - zero-cost dispatch
			| "storage:onDidChangeItems"
			| "storage:logStorage"
			| "configuration:onDidChange"
			| "workspaces:onDidChangeWorkspaceFolders"
			| "workspaces:onDidChangeWorkspaceName"
			// Command registry stubs
			| "commands:registerCommand"
			| "commands:unregisterCommand"
			| "commands:onDidRegisterCommand"
			| "commands:onDidExecuteCommand"
	);

	let OTLPStart = if IsHighFrequencyCommand { 0 } else { crate::IPC::DevLog::NowNano::Fn() };

	// Silence the per-call invoke log for high-frequency methods that are
	// not useful in forensic review. The workbench emits thousands of
	// `logger:log` invocations per boot (every `console.*` call inside VS
	// Code code becomes an IPC round-trip); keeping those lines only
	// expands log volume without adding signal. The actual dispatch below
	// still runs - this just skips the `[DEV:IPC] invoke:` line.

	if !IsHighFrequencyCommand {
		dev_log!("ipc", "invoke: {} args_count={}", command, Arguments.len());
	}

	// Ensure userdata directories exist on first IPC call
	ensure_userdata_dirs();

	// Get the application RunTime - deref the Tauri State into an owned Arc
	// so we can hand it to an Echo scheduler task below (State<T> isn't
	// Send across task boundaries).
	let RunTime:Arc<ApplicationRunTime> = ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

	// Short-circuit known no-op commands BEFORE Echo scheduler submission
	// to avoid oneshot channel allocation, String clone, and scheduler
	// overhead for calls that return Ok(Value::Null) unconditionally.
	// These account for the bulk of high-frequency IPC traffic (logger,
	// file watch, storage events, command registration).
	if IsHighFrequencyCommand {
		match command.as_str() {

			// Logger: forward error/warn/critical to dev_log; drop the rest.
			// `logger:log` (info/debug/trace) fires thousands of times per boot
			// from VS Code console.* calls - we gate those to `vscode-log`
			// which is opt-in. Errors and warnings are always surfaced.
			"logger:error" | "logger:critical" => {

				let Msg = Arguments.get(1).and_then(|V| V.as_str()).unwrap_or(
					Arguments.first().and_then(|V| V.as_str()).unwrap_or(""),
				);

				if !Msg.is_empty() {

					dev_log!("vscode-log", "[ERROR] {}", Msg);
				}

				return Ok(Value::Null);
			},

			"logger:warn" => {

				let Msg = Arguments.get(1).and_then(|V| V.as_str()).unwrap_or(
					Arguments.first().and_then(|V| V.as_str()).unwrap_or(""),
				);

				if !Msg.is_empty() {

					dev_log!("vscode-log", "[WARN] {}", Msg);
				}

				return Ok(Value::Null);
			},

			"logger:log" | "logger:info" | "logger:debug" | "logger:trace"
			| "logger:flush" | "logger:setLevel" | "logger:getLevel"
			| "logger:createLogger" | "logger:registerLogger"
			| "logger:deregisterLogger" | "logger:getRegisteredLoggers"
			| "logger:setVisibility"
			// Legacy log-service stubs: VS Code 1.87+ calls `log:registerLogger`
			// / `log:createLogger` (short prefix) in addition to the `logger:*`
			// family. Both are registered in Channel.rs so the "registered but no
			// dispatch arm" error fired on every boot. Stub-ack here alongside the
			// logger:* group.
			| "log:registerLogger" | "log:createLogger"
			// Storage event stubs: change delivery via Tauri events
			| "storage:onDidChangeItems" | "storage:logStorage"
			// Command registry stubs: side effects handled via gRPC
			| "commands:registerCommand" | "commands:unregisterCommand"
			| "commands:onDidRegisterCommand" | "commands:onDidExecuteCommand"
			// Configuration event stub
			| "configuration:onDidChange"
			// Storage lifecycle stubs
			| "storage:optimize" | "storage:isUsed" | "storage:close"
			// Workspace event stubs: change delivery via Tauri events
			| "workspaces:onDidChangeWorkspaceFolders"
			| "workspaces:onDidChangeWorkspaceName" => {

				return Ok(Value::Null);
			},

			// Menubar: acknowledged with atomic counter in the Echo path,
			// but fast-path here to save scheduler overhead per call.
			"menubar:updateMenubar" => {

				use std::sync::atomic::{AtomicU64, Ordering as AO};

				static MENUBAR_CALLS_FAST:AtomicU64 = AtomicU64::new(0);

				let N = MENUBAR_CALLS_FAST.fetch_add(1, AO::Relaxed) + 1;

				if N == 1 || N % 100 == 0 {

					dev_log!("menubar", "menubar:updateMenubar (fast-path call #{})", N);
				}

				return Ok(Value::Null);
			},

			_ => {}, // fall through to Echo dispatch for real work
		}
	}

	// Tag the pending IPC with its priority lane and submit the entire
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

	let Scheduler = RunTime.Scheduler.clone();

	let (ResultSender, ResultReceiver) = tokio::sync::oneshot::channel::<Result<Value, String>>();

	let DispatchAppHandle = ApplicationHandle.clone();

	let DispatchRuntime = RunTime.clone();

	let DispatchCommand = command.clone();

	let DispatchArgs = Arguments;

	Scheduler.Submit(
		async move {
			let ApplicationHandle = DispatchAppHandle;
			let RunTime = DispatchRuntime;
			let command = DispatchCommand;
			let Arguments = DispatchArgs;

			let MatchResult = match command.as_str() {
				// Configuration commands. VS Code's stock
				// `ConfigurationService` channel calls `getValue` /
				// `updateValue`; Mountain's native Effect-TS layer calls
				// `get` / `update`. Alias both to the same handler so
				// traffic from either rail lands in the same place.
				"configuration:get" | "configuration:getValue" => call!(rt, "config", ConfigurationGet, Arguments),
				"configuration:update" | "configuration:updateValue" => {
					call!(rt, "config", ConfigurationUpdate, Arguments)
				},
				// `ConfigurationService` listens for `onDidChange` from
				// the channel on the binary IPC rail. Mountain broadcasts
				// config changes via a Tauri event directly; ack the
				// channel-listen with Null so the ChannelClient doesn't
				// leak a pending promise.
				"configuration:onDidChange" => Ok(Value::Null),

				// `configuration:lookup` is VS Code's
				// `IConfigurationService.getValue(key)` called from the
				// workbench's `ConfigurationService` singleton. Wire shape is
				// identical to `configuration:get`; alias so both rails resolve
				// the same underlying value.
				"configuration:lookup" => {
					call!(rt, "config", "configuration:lookup (→ get)", ConfigurationGet, Arguments)
				},

				// `configuration:inspect` is `IConfigurationService.inspect(key)`.
				// The workbench destructures `{ value, default, user, workspace,
				// workspaceFolder }` from the result unconditionally; returning a
				// plain value or null crashes the Settings UI. We surface the
				// current effective value in both `value` and `default` (since
				// Mountain has no per-scope override layer yet) and null for the
				// remaining scopes. VS Code treats null scopes as "not set",
				// which is correct for Land where no user/workspace JSON overrides
				// exist.
				"configuration:inspect" => {
					dev_log!("config", "configuration:inspect");
					let CurrentValue = ConfigurationGet(RunTime.clone(), Arguments).await.unwrap_or(Value::Null);
					Ok(json!({
						"value": CurrentValue,
						"default": CurrentValue,
						"user": Value::Null,
						"workspace": Value::Null,
						"workspaceFolder": Value::Null,
						"memory": Value::Null,
					}))
				},

				// Logger commands - all logger:* are high-frequency and handled in the
				// fast-path short-circuit above. These Echo arms are only reached
				// if IS_HIGH_FREQUENCY detection changes; they provide the same
				// dev_log output as the fast-path for safety.
				"logger:log" | "logger:warn" | "logger:error" | "logger:info" | "logger:debug" | "logger:trace" => {
					let Level = command.trim_start_matches("logger:");
					let Msg = if Arguments.len() >= 2 {
						let Tail:Vec<String> = Arguments
							.iter()
							.skip(1)
							.filter_map(|V| V.as_str().map(str::to_owned).or_else(|| serde_json::to_string(V).ok()))
							.collect();
						Tail.join(" ")
					} else {
						Arguments
							.first()
							.and_then(|V| V.as_str().map(str::to_owned))
							.unwrap_or_default()
					};
					if !Msg.is_empty() {
						match Level {
							"error" | "critical" => dev_log!("vscode-log", "[ERROR] {}", Msg),
							"warn" => dev_log!("vscode-log", "[WARN] {}", Msg),
							_ => dev_log!("vscode-log", "{}", Msg),
						}
					}
					Ok(Value::Null)
				},
				"logger:flush"
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
				"file:read" | "file:readFile" => FileReadNative(Arguments).await,
				"file:write" | "file:writeFile" => FileWriteNative(Arguments).await,
				"file:stat" => FileStatNative(Arguments).await,
				"file:exists" => FileExistsNative(Arguments).await,
				"file:delete" => FileDeleteNative(Arguments).await,
				"file:copy" => FileCloneNative(Arguments).await,
				"file:move" | "file:rename" => FileRenameNative(Arguments).await,
				"file:mkdir" => FileMkdirNative(Arguments).await,
				"file:readdir" => FileReaddirNative(Arguments).await,
				"file:readBinary" => FileReadBinary(RunTime.clone(), Arguments).await,
				"file:writeBinary" => FileWriteBinary(RunTime.clone(), Arguments).await,
				// File watcher channel methods - `DiskFileSystemProvider`
				// opens `watch` / `unwatch` channel calls to receive
				"file:watch" => FileWatch(RunTime.clone(), Arguments).await,
				"file:unwatch" => FileUnwatch(RunTime.clone(), Arguments).await,

				// Storage commands. VS Code's
				// `ApplicationStorageDatabaseClient` channel methods are
				// `getItems` / `updateItems` / `optimize` / `close` /
				// `isUsed`; the shorter `storage:get` / `storage:set` are
				// Mountain-native conveniences. All route through the
				// same ApplicationState storage backing.
				"storage:get" => StorageGet(RunTime.clone(), Arguments).await,
				"storage:set" => StorageSet(RunTime.clone(), Arguments).await,
				// Workbench services poll this on every theme / scope
				// change; suppress the bare banner and rely on the IPC
				// `invoke:`/`done:` summary for volume + latency.
				"storage:getItems" => call!(rt, "storage-verbose", "storage:getItems", StorageGetItems, Arguments),
				"storage:updateItems" => {
					call!(rt, "storage-verbose", "storage:updateItems", StorageUpdateItems, Arguments)
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
				"environment:get" => call!(rt, "config", "environment:get", EnvironmentGet, Arguments),

				// Native host commands
				"native:showItemInFolder" => ShowItemInFolder(RunTime.clone(), Arguments).await,
				"native:openExternal" => OpenExternal(RunTime.clone(), Arguments).await,

				// Workbench commands
				"workbench:getConfiguration" => WorkbenchConfiguration(RunTime.clone(), Arguments).await,

				// Diagnostic: webview → Mountain dev-log bridge.
				// First arg is a tag ("boot", "extService", …), second is the
				// message, rest are optional structured fields we stringify.
				// Atom H1c: added so workbench.js can surface diagnostic state
				// into the same Mountain.dev.log that carries Rust-side events.
				"diagnostic:log" => {
					let Tag = arg_string_or(&Arguments, 0, "webview");
					let Message = arg_string(&Arguments, 1);
					let Extras = if Arguments.len() > 2 {
						let Tail:Vec<String> = Arguments
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
				"commands:execute" | "commands:executeCommand" => CommandsExecute(RunTime.clone(), Arguments).await,
				"commands:getAll" | "commands:getCommands" => {
					dev_log!("commands", "{}", command);
					CommandsGetAll(RunTime.clone()).await
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
					ExtensionsGetAll(RunTime.clone()).await
				},
				"extensions:get" => {
					dev_log!("extensions", "extensions:get");
					ExtensionsGet(RunTime.clone(), Arguments).await
				},
				"extensions:isActive" => {
					dev_log!("extensions", "extensions:isActive");
					ExtensionsIsActive(RunTime.clone(), Arguments).await
				},
				// `extensions:activate(extensionId)` - send `$activateByEvent`
				// to Cocoon so the extension host starts the extension. VS Code
				// normally drives activation via the workbench's activation events
				// (onStartupFinished, onLanguage:*, etc.); this path lets Wind's
				// ExtensionsService trigger activation programmatically.
				"extensions:activate" => {
					let ExtensionId = arg_string(&Arguments, 0);
					dev_log!("extensions", "extensions:activate id={}", ExtensionId);
					if ExtensionId.is_empty() {
						Ok(Value::Null)
					} else {
						let Notification = json!({
							"event": format!("onCustom:{}", ExtensionId),
							"extensionId": ExtensionId,
						});
						let _ = crate::Vine::Client::SendNotification::Fn(
							"cocoon-main".to_string(),
							"$activateByEvent".to_string(),
							Notification,
						)
						.await;
						Ok(Value::Null)
					}
				},

				// VS Code's Extensions sidebar →
				// `ExtensionManagementChannelClient.getInstalled` goes through
				// `sharedProcessService.getChannel('extensions')`. Sky's
				// astro.config.ts Step 7b swaps the native SharedProcessService
				// for a TauriMainProcessService-backed shim, so the call lands
				// here as `extensions:getInstalled`. The expected return is
				// `ILocalExtension[]` - a wrapper around each scanned manifest
				// with `identifier.id`, `manifest`, `location`, `isBuiltin`, etc.
				// `ExtensionsGetInstalled` builds that envelope;
				// `ExtensionsGetAll` returns the raw manifest for
				// callers (Cocoon, Wind Effect services) that want the flat
				// shape. Do NOT alias these two - the payload shapes differ.
				"extensions:getInstalled" | "extensions:scanSystemExtensions" => {
					// Atom H1a: Arguments[0]=type, Arguments[1]=profileLocation URI,
					// Arguments[2]=productVersion, Arguments[3]=??? (VS Code canonical is
					// 3; shim appears to add a 4th). Dump to find out what it
					// contains on post-nav page reloads where the sidebar
					// renders 0 entries despite Mountain returning 94.
					let ArgsSummary = Arguments
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
					dev_log!("extensions", "{} Arguments={}", command, ArgsSummary);
					// `scanSystemExtensions` is conceptually
					// `getInstalled(type=ExtensionType.System)`, so override
					// `Arguments[0]` to `0` before forwarding. Without the override
					// a plain alias would inherit whatever the caller passed
					// in Arguments[0] (which for the VS Code channel client is
					// usually `null`) and leak User extensions into the
					// System list - the same bug we just fixed at the
					// handler layer, one level up.
					let EffectiveArgs = if command == "extensions:scanSystemExtensions" {
						let mut Overridden = Arguments.clone();
						if Overridden.is_empty() {
							Overridden.push(Value::Null);
						}
						Overridden[0] = json!(0);
						Overridden
					} else {
						Arguments.clone()
					};
					ExtensionsGetInstalled(RunTime.clone(), EffectiveArgs).await
				},
				"extensions:scanUserExtensions" => {
					// User-scope scan. Forward to the unified handler with
					// `type=ExtensionType.User (1)` so VSIX-installed
					// extensions under `~/.fiddee/extensions/*` come back
					// even when the caller didn't pass an explicit type
					// filter (VS Code's channel client does that on
					// scan-user-extensions, which is why the sidebar
					// previously saw an empty list after every
					// Install-from-VSIX).
					dev_log!("extensions", "{} (forwarded to getInstalled with type=User)", command);
					let mut UserArgs = Arguments.clone();
					if UserArgs.is_empty() {
						UserArgs.push(Value::Null);
					}
					UserArgs[0] = json!(1);
					ExtensionsGetInstalled(RunTime.clone(), UserArgs).await
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
					Extension::ExtensionInstall::Fn(ApplicationHandle.clone(), RunTime.clone(), Arguments).await
				},
				"extensions:uninstall" => {
					Extension::ExtensionUninstall::Fn(ApplicationHandle.clone(), RunTime.clone(), Arguments).await
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
					let VsixPath = match Arguments.first() {
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
				"terminal:create" => call!(rt, "terminal", "terminal:create", TerminalCreate, Arguments),
				"terminal:sendText" => call!(rt, "terminal", "terminal:sendText", TerminalSendText, Arguments),
				"terminal:dispose" => call!(rt, "terminal", "terminal:dispose", TerminalDispose, Arguments),
				"terminal:show" => call!(rt, "terminal", "terminal:show", TerminalShow, Arguments),
				"terminal:hide" => call!(rt, "terminal", "terminal:hide", TerminalHide, Arguments),

				// Output channel commands
				"output:create" => OutputCreate(ApplicationHandle.clone(), Arguments).await,
				"output:append" => call!(app, "output", "output:append", OutputAppend, Arguments),
				"output:appendLine" => call!(app, "output", "output:appendLine", OutputAppendLine, Arguments),
				"output:clear" => call!(app, "output", "output:clear", OutputClear, Arguments),
				"output:show" => call!(app, "output", "output:show", OutputShow, Arguments),

				// TextFile commands
				"textFile:read" => call!(rt, "textfile", "textFile:read", TextfileRead, Arguments),
				"textFile:write" => call!(rt, "textfile", "textFile:write", TextfileWrite, Arguments),
				"textFile:save" => TextfileSave(RunTime.clone(), Arguments).await,

				// Storage commands (additional)
				"storage:delete" => call!(rt, "storage", "storage:delete", StorageDelete, Arguments),
				"storage:keys" => call!(rt, "storage", "storage:keys", StorageKeys),

				// Notification commands (emit sky:// events for Sky to render)
				"notification:show" => call!(app, "notification", "notification:show", NotificationShow, Arguments),
				"notification:showProgress" => {
					call!(
						app,
						"notification",
						"notification:showProgress",
						NotificationShowProgress,
						Arguments
					)
				},
				"notification:updateProgress" => {
					call!(
						app,
						"notification",
						"notification:updateProgress",
						NotificationUpdateProgress,
						Arguments
					)
				},
				"notification:endProgress" => {
					call!(
						app,
						"notification",
						"notification:endProgress",
						NotificationEndProgress,
						Arguments
					)
				},

				// Progress commands
				"progress:begin" => call!(app, "progress", "progress:begin", ProgressBegin, Arguments),
				"progress:report" => call!(app, "progress", "progress:report", ProgressReport, Arguments),
				"progress:end" => call!(app, "progress", "progress:end", ProgressEnd, Arguments),

				// QuickInput commands
				"quickInput:showQuickPick" => {
					call!(rt, "quickinput", "quickInput:showQuickPick", QuickInputShowQuickPick, Arguments)
				},
				"quickInput:showInputBox" => {
					call!(rt, "quickinput", "quickInput:showInputBox", QuickInputShowInputBox, Arguments)
				},

				// Workspaces commands. VS Code's `IWorkspacesService`
				// channel uses `getWorkspaceFolders` /
				// `addWorkspaceFolders`; Mountain's rail uses the
				// shorter `getFolders` / `addFolder`. Alias both.
				"workspaces:getFolders" | "workspaces:getWorkspaceFolders" | "workspaces:getWorkspace" => {
					call!(rt, "workspaces", WorkspacesGetFolders)
				},
				"workspaces:addFolder" | "workspaces:addWorkspaceFolders" => {
					call!(rt, "workspaces", WorkspacesAddFolder, Arguments)
				},
				"workspaces:removeFolder" | "workspaces:removeWorkspaceFolders" => {
					call!(rt, "workspaces", WorkspacesRemoveFolder, Arguments)
				},
				"workspaces:getName" => call!(rt, "workspaces", WorkspacesGetName),
				// Themes commands
				"themes:getActive" => call!(rt, "themes", "themes:getActive", ThemesGetActive),
				"themes:list" => call!(rt, "themes", "themes:list", ThemesList),
				"themes:set" => call!(rt, "themes", "themes:set", ThemesSet, Arguments),
				// `IThemeService.getColorTheme()` - workbench channel method used
				// by tokenization, decoration, and the colour-picker to read the
				// active theme object. Wire shape differs from `themes:getActive`
				// only in name; alias to the same handler.
				"themes:getColorTheme" => call!(rt, "themes", "themes:getColorTheme (→ getActive)", ThemesGetActive),

				// Search commands. Stock VS Code `SearchService` channel
				// uses `textSearch` / `fileSearch`; Mountain's Effect-TS
				// rail uses `findInFiles` / `findFiles`. Alias both.
				"search:findInFiles" | "search:textSearch" | "search:searchText" => {
					call!(rt, "search", SearchFindInFiles, Arguments)
				},
				"search:findFiles" | "search:fileSearch" | "search:searchFile" => {
					call!(rt, "search", SearchFindFiles, Arguments)
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
					DecorationsGet(RunTime.clone(), Arguments).await
				},
				"decorations:getMany" => {
					dev_log!("decorations", "decorations:getMany");
					DecorationsGetMany(RunTime.clone(), Arguments).await
				},
				"decorations:set" => {
					dev_log!("decorations", "decorations:set");
					DecorationsSet(RunTime.clone(), Arguments).await
				},
				"decorations:clear" => {
					dev_log!("decorations", "decorations:clear");
					DecorationsClear(RunTime.clone(), Arguments).await
				},

				// WorkingCopy commands
				"workingCopy:isDirty" => {
					dev_log!("workingcopy", "workingCopy:isDirty");
					WorkingCopyIsDirty(RunTime.clone(), Arguments).await
				},
				"workingCopy:setDirty" => {
					dev_log!("workingcopy", "workingCopy:setDirty");
					WorkingCopySetDirty(RunTime.clone(), Arguments).await
				},
				"workingCopy:getAllDirty" => {
					dev_log!("workingcopy", "workingCopy:getAllDirty");
					WorkingCopyGetAllDirty(RunTime.clone()).await
				},
				"workingCopy:getDirtyCount" => {
					dev_log!("workingcopy", "workingCopy:getDirtyCount");
					WorkingCopyGetDirtyCount(RunTime.clone()).await
				},

				// Keybinding commands
				"keybinding:add" => {
					dev_log!("keybinding", "keybinding:add");
					KeybindingAdd(RunTime.clone(), Arguments).await
				},
				"keybinding:remove" => {
					dev_log!("keybinding", "keybinding:remove");
					KeybindingRemove(RunTime.clone(), Arguments).await
				},
				"keybinding:lookup" => {
					dev_log!("keybinding", "keybinding:lookup");
					KeybindingLookup(RunTime.clone(), Arguments).await
				},
				"keybinding:getAll" => {
					dev_log!("keybinding", "keybinding:getAll");
					KeybindingGetAll(RunTime.clone()).await
				},

				// Lifecycle commands
				"lifecycle:getPhase" => {
					dev_log!("lifecycle", "lifecycle:getPhase");
					LifecycleGetPhase(RunTime.clone()).await
				},
				"lifecycle:whenPhase" => {
					dev_log!("lifecycle", "lifecycle:whenPhase");
					LifecycleWhenPhase(RunTime.clone(), Arguments).await
				},
				"lifecycle:requestShutdown" => {
					dev_log!("lifecycle", "lifecycle:requestShutdown");
					LifecycleRequestShutdown(ApplicationHandle.clone()).await
				},
				"lifecycle:advancePhase" | "lifecycle:setPhase" => {
					dev_log!("lifecycle", "{}", command);
					// Wind calls this at the end of every workbench init pass so
					// the phase advances Starting → Ready → Restored → Eventually.
					// Mountain emits `sky://lifecycle/phaseChanged` so any extension
					// host or service waiting on a later phase wakes up.
					let NewPhase = arg_u64_or(&Arguments, 0, 1) as u8;
					RunTime
						.Environment
						.ApplicationState
						.Feature
						.Lifecycle
						.AdvanceAndBroadcast(NewPhase, &ApplicationHandle);

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
						if let Some(MainWindow) = ApplicationHandle.get_webview_window("main") {
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

					Ok(json!(RunTime.Environment.ApplicationState.Feature.Lifecycle.GetPhase()))
				},

				// Label commands
				"label:getUri" => {
					dev_log!("label", "label:getUri");
					LabelGetURI(RunTime.clone(), Arguments).await
				},
				"label:getWorkspace" => {
					dev_log!("label", "label:getWorkspace");
					LabelGetWorkspace(RunTime.clone()).await
				},
				"label:getBase" => {
					dev_log!("label", "label:getBase");
					LabelGetBase(Arguments).await
				},

				// Model (text model registry) commands
				"model:open" => {
					dev_log!("model", "model:open");
					ModelOpen(RunTime.clone(), Arguments).await
				},
				"model:close" => {
					dev_log!("model", "model:close");
					ModelClose(RunTime.clone(), Arguments).await
				},
				"model:get" => {
					dev_log!("model", "model:get");
					ModelGet(RunTime.clone(), Arguments).await
				},
				"model:getAll" => {
					dev_log!("model", "model:getAll");
					ModelGetAll(RunTime.clone()).await
				},
				"model:updateContent" => {
					dev_log!("model", "model:updateContent");
					ModelUpdateContent(RunTime.clone(), Arguments).await
				},

				// Navigation history commands
				"history:goBack" => {
					dev_log!("history", "history:goBack");
					HistoryGoBack(RunTime.clone()).await
				},
				"history:goForward" => {
					dev_log!("history", "history:goForward");
					HistoryGoForward(RunTime.clone()).await
				},
				"history:canGoBack" => {
					dev_log!("history", "history:canGoBack");
					HistoryCanGoBack(RunTime.clone()).await
				},
				"history:canGoForward" => {
					dev_log!("history", "history:canGoForward");
					HistoryCanGoForward(RunTime.clone()).await
				},
				"history:push" => {
					dev_log!("history", "history:push");
					HistoryPush(RunTime.clone(), Arguments).await
				},
				"history:clear" => {
					dev_log!("history", "history:clear");
					HistoryClear(RunTime.clone()).await
				},
				"history:getStack" => {
					dev_log!("history", "history:getStack");
					HistoryGetStack(RunTime.clone()).await
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
					// Return the live merged configuration object.
					let Config = RunTime.Environment.ApplicationState.Configuration.GetGlobalConfiguration();
					Ok(Config)
				},
				"mountain_get_services_status" => {
					let CocoonConnected = crate::Vine::Client::IsClientConnected::Fn("cocoon-main");
					Ok(json!({
						"cocoon": { "connected": CocoonConnected },
						"vine": { "running": true }
					}))
				},
				"mountain_get_state" => {
					let FolderCount = RunTime
						.Environment
						.ApplicationState
						.Workspace
						.WorkspaceFolders
						.lock()
						.map(|G| G.len())
						.unwrap_or(0);
					Ok(json!({
						"workspace": { "folderCount": FolderCount },
						"activeDocument": RunTime.Environment.ApplicationState.Workspace.GetActiveDocumentURI()
					}))
				},

				// =====================================================================
				// File system command ALIASES
				// VS Code's DiskFileSystemProviderClient calls readFile/writeFile/rename
				// but Mountain's original handlers use read/write/move.
				// =====================================================================
				"file:realpath" => FileRealpath(Arguments).await,
				"file:open" => FileOpenFd(Arguments).await,
				"file:close" => FileCloseFd(Arguments).await,
				"file:cloneFile" => FileCloneNative(Arguments).await,

				// =====================================================================
				// Native Host commands (INativeHostService)
				// =====================================================================

				// Dialogs
				"nativeHost:pickFolderAndOpen" => NativePickFolder(ApplicationHandle.clone(), Arguments).await,
				"nativeHost:pickFileAndOpen" => NativePickFolder(ApplicationHandle.clone(), Arguments).await,
				"nativeHost:pickFileFolderAndOpen" => NativePickFolder(ApplicationHandle.clone(), Arguments).await,
				"nativeHost:pickWorkspaceAndOpen" => NativePickFolder(ApplicationHandle.clone(), Arguments).await,
				"nativeHost:showOpenDialog" => NativeShowOpenDialog(ApplicationHandle.clone(), Arguments).await,

				// Wind's `Files/Live.ts` calls `UserInterface.ShowOpenDialog` via
				// IPC and expects a bare `string[]` (file paths). The
				// `NativeShowOpenDialog` handler returns `{ canceled, filePaths }`.
				// Unwrap here so Wind's `Array.isArray(Result) ? Result : []`
				// finds the array rather than silently falling back to `[]`.
				"UserInterface.ShowOpenDialog" => {
					match NativeShowOpenDialog(ApplicationHandle.clone(), Arguments).await {
						Ok(Response) => {
							let Paths = Response
								.get("filePaths")
								.and_then(|V| V.as_array())
								.cloned()
								.unwrap_or_default();
							Ok(Value::Array(Paths))
						},
						Err(Error) => Err(Error),
					}
				},
				"nativeHost:showSaveDialog" => NativeShowSaveDialog(ApplicationHandle.clone(), Arguments).await,
				// Wind's `Files/Live.ts` calls `UserInterface.ShowSaveDialog` via
				// IPC and expects a bare path string (or undefined).
				"UserInterface.ShowSaveDialog" => {
					UserInterfaceShowSaveDialog(ApplicationHandle.clone(), Arguments).await
				},
				"nativeHost:showMessageBox" => NativeShowMessageBox(ApplicationHandle.clone(), Arguments).await,

				// Environment paths - delegated to atomic handler.
				"nativeHost:getEnvironmentPaths" => NativeGetEnvironmentPaths(ApplicationHandle.clone()).await,

				// OS info
				"nativeHost:getOSColorScheme" => {
					dev_log!("nativehost", "nativeHost:getOSColorScheme");
					NativeGetColorScheme().await
				},
				"nativeHost:getOSProperties" => {
					dev_log!("nativehost", "nativeHost:getOSProperties");
					NativeOSProperties().await
				},
				"nativeHost:getOSStatistics" => {
					dev_log!("nativehost", "nativeHost:getOSStatistics");
					NativeOSStatistics().await
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
					NativeIsFullscreen(ApplicationHandle.clone()).await
				},
				"nativeHost:isMaximized" => {
					dev_log!("window", "nativeHost:isMaximized");
					NativeIsMaximized(ApplicationHandle.clone()).await
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
					// Cursor position is used by the workbench to bias overlay
					// placement. (0,0) causes overlays to appear at the top-left
					// and get clipped to sane positions - zero overhead vs
					// spawning an osascript process per call.
					Ok(json!({ "x": 0, "y": 0 }))
				},
				"nativeHost:getWindows" => {
					let Title = std::env::var("ProductNameShort").unwrap_or_else(|_| "Land".into());
					let ActiveDoc = RunTime
						.Environment
						.ApplicationState
						.Workspace
						.GetActiveDocumentURI()
						.unwrap_or_default();
					Ok(json!([{ "id": 1, "title": Title, "filename": ActiveDoc }]))
				},
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
					if let Some(Window) = ApplicationHandle.get_webview_window("main") {
						let _ = Window.set_focus();
					}
					Ok(Value::Null)
				},
				"nativeHost:maximizeWindow" => {
					dev_log!("window", "{}", command);
					if let Some(Window) = ApplicationHandle.get_webview_window("main") {
						let _ = Window.maximize();
					}
					Ok(Value::Null)
				},
				"nativeHost:unmaximizeWindow" => {
					dev_log!("window", "{}", command);
					if let Some(Window) = ApplicationHandle.get_webview_window("main") {
						let _ = Window.unmaximize();
					}
					Ok(Value::Null)
				},
				"nativeHost:minimizeWindow" => {
					dev_log!("window", "{}", command);
					if let Some(Window) = ApplicationHandle.get_webview_window("main") {
						let _ = Window.minimize();
					}
					Ok(Value::Null)
				},
				"nativeHost:toggleFullScreen" => {
					dev_log!("window", "{}", command);
					if let Some(Window) = ApplicationHandle.get_webview_window("main") {
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
					if let Some(Window) = ApplicationHandle.get_webview_window("main") {
						let _ = Window.destroy();
					}
					Ok(Value::Null)
				},
				"nativeHost:setWindowAlwaysOnTop" => {
					dev_log!("window", "{}", command);
					let OnTop = arg_bool(&Arguments, 0);
					if let Some(Window) = ApplicationHandle.get_webview_window("main") {
						let _ = Window.set_always_on_top(OnTop);
					}
					Ok(Value::Null)
				},
				"nativeHost:toggleWindowAlwaysOnTop" => {
					dev_log!("window", "{}", command);
					static ALWAYS_ON_TOP:std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
					let Next = !ALWAYS_ON_TOP.fetch_xor(true, std::sync::atomic::Ordering::Relaxed);
					if let Some(Window) = ApplicationHandle.get_webview_window("main") {
						let _ = Window.set_always_on_top(Next);
					}
					Ok(Value::Null)
				},
				// `NSWindow.representedFilename` - sets the proxy icon in the
				// macOS title bar. Tauri doesn't expose this directly; use
				// Window.set_title as a best-effort (shows path in title).
				"nativeHost:setRepresentedFilename" => {
					dev_log!("window", "{}", command);
					let Path = arg_string(&Arguments, 0);
					if !Path.is_empty() {
						if let Some(Window) = ApplicationHandle.get_webview_window("main") {
							// Show just the filename component as the title; the
							// full path would overflow the title bar on deep trees.
							let Filename = std::path::Path::new(&Path)
								.file_name()
								.and_then(|N| N.to_str())
								.unwrap_or(&Path);
							let _ = Window.set_title(Filename);
						}
					}
					Ok(Value::Null)
				},

				// `NSWindow.isDocumentEdited` - the ● dirty dot in the macOS
				// title bar. NSWindow::setDocumentEdited is not exposed by
				// Tauri 2.x's WebviewWindow API; acknowledged as no-op.
				"nativeHost:setDocumentEdited" => {
					let _ = Arguments;
					Ok(Value::Null)
				},

				// `nativeHost:setMinimumSize` - enforce a minimum window size so
				// the workbench never collapses to a 1×1 pixel frame.
				"nativeHost:setMinimumSize" => {
					let Width = arg_u64_or(&Arguments, 0, 400) as u32;
					let Height = arg_u64_or(&Arguments, 1, 300) as u32;
					if let Some(Window) = ApplicationHandle.get_webview_window("main") {
						let _ = Window.set_min_size(Some(tauri::Size::Physical(tauri::PhysicalSize {
							width:Width,
							height:Height,
						})));
					}
					Ok(Value::Null)
				},

				// `nativeHost:positionWindow` - move the window to an explicit
				// screen position (used by multi-window restore).
				"nativeHost:positionWindow" => {
					if let Some(Rect) = Arguments.first() {
						let X = Rect.get("x").and_then(|V| V.as_i64()).unwrap_or(0) as i32;
						let Y = Rect.get("y").and_then(|V| V.as_i64()).unwrap_or(0) as i32;
						let W = Rect.get("width").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
						let H = Rect.get("height").and_then(|V| V.as_u64()).unwrap_or(0) as u32;
						if let Some(Window) = ApplicationHandle.get_webview_window("main") {
							let _ =
								Window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x:X, y:Y }));
							if W > 0 && H > 0 {
								let _ =
									Window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width:W, height:H }));
							}
						}
					}
					Ok(Value::Null)
				},

				// Pure lifecycle/cosmetic signals - no Mountain-side action needed.
				"nativeHost:updateWindowControls"
				| "nativeHost:notifyReady"
				| "nativeHost:saveWindowSplash"
				| "nativeHost:updateTouchBar"
				| "nativeHost:moveWindowTop"
				| "nativeHost:setBackgroundThrottling"
				| "nativeHost:updateWindowAccentColor" => {
					dev_log!("window", "{}", command);
					Ok(Value::Null)
				},

				// OS operations
				"nativeHost:isAdmin" => Ok(json!(false)),
				"nativeHost:isRunningUnderARM64Translation" => NativeIsRunningUnderARM64Translation().await,
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
				"nativeHost:showItemInFolder" => ShowItemInFolder(RunTime.clone(), Arguments).await,
				"nativeHost:openExternal" => OpenExternal(RunTime.clone(), Arguments).await,
				// Trash bin - atomic handler handles all platform variants.
				"nativeHost:moveItemToTrash" => {
					dev_log!("nativehost", "nativeHost:moveItemToTrash");
					NativeMoveItemToTrash(Arguments).await
				},

				// Clipboard - atomic handlers backed by `arboard`.
				"nativeHost:readClipboardText" => {
					dev_log!("clipboard", "readClipboardText");
					NativeReadClipboardText(Arguments).await
				},
				"nativeHost:writeClipboardText" => {
					dev_log!("clipboard", "writeClipboardText");
					NativeWriteClipboardText(Arguments).await
				},
				"nativeHost:readClipboardFindText" => {
					dev_log!("clipboard", "readClipboardFindText");
					NativeReadClipboardFindText(Arguments).await
				},
				"nativeHost:writeClipboardFindText" => {
					dev_log!("clipboard", "writeClipboardFindText");
					NativeWriteClipboardFindText(Arguments).await
				},
				"nativeHost:readClipboardBuffer" => {
					dev_log!("clipboard", "readClipboardBuffer");
					NativeReadClipboardBuffer(Arguments).await
				},
				"nativeHost:writeClipboardBuffer" => {
					dev_log!("clipboard", "writeClipboardBuffer");
					NativeWriteClipboardBuffer(Arguments).await
				},
				"nativeHost:hasClipboard" => {
					dev_log!("clipboard", "hasClipboard");
					NativeHasClipboard(Arguments).await
				},
				"nativeHost:readImage" => {
					dev_log!("clipboard", "readImage");
					NativeReadImage(Arguments).await
				},
				"nativeHost:triggerPaste" => {
					dev_log!("clipboard", "triggerPaste");
					NativeTriggerPaste(Arguments).await
				},

				// Process
				"nativeHost:getProcessId" => Ok(json!(std::process::id())),
				"nativeHost:killProcess" => KillProcess(Arguments).await,

				// Network
				"nativeHost:findFreePort" => NativeFindFreePort(Arguments).await,
				"nativeHost:isPortFree" => {
					let Port = arg_u64(&Arguments, 0) as u16;
					if Port == 0 {
						Ok(json!(false))
					} else {
						let Free = tokio::net::TcpListener::bind(std::net::SocketAddr::from(([127, 0, 0, 1], Port)))
							.await
							.is_ok();
						Ok(json!(Free))
					}
				},
				// `IProxyService.resolveProxy` - return `DIRECT` when no proxy
				// env var is set, or the var's value when one is configured.
				// VS Code uses this before every authenticated HTTP request so
				// extensions that call `fetch` route through the right gateway.
				"nativeHost:resolveProxy" => {
					let Url = arg_str(&Arguments, 0);
					let Scheme = if Url.starts_with("https") { "HTTPS" } else { "HTTP" };
					let ProxyEnv = std::env::var(format!("{}_PROXY", Scheme))
						.or_else(|_| std::env::var(format!("{}_proxy", Scheme.to_lowercase())))
						.or_else(|_| std::env::var("ALL_PROXY"))
						.or_else(|_| std::env::var("all_proxy"));
					match ProxyEnv {
						Ok(P) if !P.is_empty() => {
							// Strip scheme and emit the correct PAC keyword.
							// socks/socks4/socks5 → "SOCKS host:port" (RFC 3513)
							// http/https          → "PROXY host:port"
							let Lower = P.to_lowercase();
							let (Keyword, Host) = if Lower.starts_with("socks") {
								let H = P
									.trim_start_matches("socks5://")
									.trim_start_matches("socks4://")
									.trim_start_matches("socks://");

								("SOCKS", H)
							} else {
								let H = P.trim_start_matches("http://").trim_start_matches("https://");

								("PROXY", H)
							};

							Ok(json!(format!("{} {}", Keyword, Host)))
						},
						_ => Ok(json!("DIRECT")),
					}
				},
				"nativeHost:lookupAuthorization" => Ok(Value::Null),
				"nativeHost:lookupKerberosAuthorization" => Ok(Value::Null),
				"nativeHost:loadCertificates" => Ok(json!([])),

				// Lifecycle
				"nativeHost:relaunch" => Relaunch(ApplicationHandle.clone(), Arguments).await,
				"nativeHost:reload" => Reload(ApplicationHandle.clone(), Arguments).await,
				"nativeHost:quit" => Quit(ApplicationHandle.clone(), Arguments).await,
				"nativeHost:exit" => Exit(ApplicationHandle.clone(), Arguments).await,

				// Dev tools
				"nativeHost:openDevTools" => OpenDevTools(ApplicationHandle.clone(), Arguments).await,
				"nativeHost:toggleDevTools" => ToggleDevTools(ApplicationHandle.clone(), Arguments).await,

				// Power
				"nativeHost:getSystemIdleState" => Ok(json!("active")),
				"nativeHost:getSystemIdleTime" => Ok(json!(0)),
				"nativeHost:getCurrentThermalState" => Ok(json!("nominal")),
				"nativeHost:isOnBatteryPower" => Ok(json!(false)),
				"nativeHost:startPowerSaveBlocker" => Ok(json!(0)),
				"nativeHost:stopPowerSaveBlocker" => Ok(json!(false)),
				"nativeHost:isPowerSaveBlockerStarted" => Ok(json!(false)),

				// Electron BrowserView management - not applicable under Tauri.
				// updateKeybindings/updateTheme/updateConfiguration are UI-state
				// notifications the renderer sends to BrowserView overlays. getBrowserViews
				// returns the list of active views. All are no-ops here.
				"browserView:updateKeybindings"
				| "browserView:updateTheme"
				| "browserView:updateConfiguration"
				| "browserView:openDevTools"
				| "browserView:closeDevTools" => Ok(Value::Null),
				"browserView:getBrowserViews" => Ok(serde_json::json!([])),

				// macOS specific
				"nativeHost:newWindowTab" => Ok(Value::Null),
				"nativeHost:showPreviousWindowTab" => Ok(Value::Null),
				"nativeHost:showNextWindowTab" => Ok(Value::Null),
				"nativeHost:moveWindowTabToNewWindow" => Ok(Value::Null),
				"nativeHost:mergeAllWindowTabs" => Ok(Value::Null),
				"nativeHost:toggleWindowTabsBar" => Ok(Value::Null),
				"nativeHost:installShellCommand" => InstallShellCommand(Arguments).await,
				"nativeHost:uninstallShellCommand" => UninstallShellCommand(Arguments).await,

				// =====================================================================
				// Local PTY (terminal) commands
				// =====================================================================
				"localPty:getProfiles" => {
					dev_log!("terminal", "localPty:getProfiles");
					LocalPTYGetProfiles().await
				},
				"localPty:getDefaultSystemShell" => {
					dev_log!("terminal", "localPty:getDefaultSystemShell");
					LocalPTYGetDefaultShell().await
				},
				// `ILocalPtyService.getTerminalLayoutInfo` - return the last
				// layout snapshot so the workbench restores the terminal panel
				// (active tab, dimensions) across window reloads.
				// Key: "terminal:layoutInfo" in Mountain's `StorageProvider`.
				// `ILocalPtyService.getTerminalLayoutInfo` - return the persisted
				// layout snapshot so the workbench restores the terminal panel
				// (active tab, split dimensions) across window reloads.
				"localPty:getTerminalLayoutInfo" => {
					dev_log!("terminal", "localPty:getTerminalLayoutInfo");
					use CommonLibrary::{Environment::Requires::Requires, Storage::StorageProvider::StorageProvider};
					let StorageProvider:Arc<dyn StorageProvider> = RunTime.Environment.Require();
					match StorageProvider.GetStorageValue(true, "terminal:layoutInfo").await {
						Ok(Some(Stored)) => Ok(Stored),
						Ok(None) => Ok(Value::Null),
						Err(Error) => {
							dev_log!("terminal", "warn: [getTerminalLayoutInfo] storage read failed: {}", Error);
							Ok(Value::Null)
						},
					}
				},
				// `ILocalPtyService.setTerminalLayoutInfo` - persist the layout
				// snapshot so `getTerminalLayoutInfo` can replay it on next boot.
				"localPty:setTerminalLayoutInfo" => {
					dev_log!("terminal", "localPty:setTerminalLayoutInfo");
					use CommonLibrary::{Environment::Requires::Requires, Storage::StorageProvider::StorageProvider};
					let StorageProvider:Arc<dyn StorageProvider> = RunTime.Environment.Require();
					let Payload = arg_val(&Arguments, 0);
					let _ = StorageProvider
						.UpdateStorageValue(true, "terminal:layoutInfo".to_string(), Some(Payload))
						.await;
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
					LocalPTYGetEnvironment().await
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
					dev_log!("ipc", "cocoon:request method={:?}", Arguments.first());
					CocoonRequest(Arguments).await
				},
				"cocoon:notify" => {
					dev_log!("ipc", "cocoon:notify method={:?}", Arguments.first());
					CocoonNotify(Arguments).await
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
				// `localPty:spawn` is Cocoon's Sky bridge path; preserve
				// the full `{id, name, pid}` shape. New `localPty:createProcess`
				// follows VS Code's typed contract.
				"localPty:spawn" => call!(rt, "terminal", TerminalCreate, Arguments),
				"localPty:createProcess" => call!(rt, "terminal", LocalPTYCreateProcess, Arguments),
				"localPty:start" => {
					// Eager-spawn pattern: `TerminalProvider::CreateTerminal`
					// already started the shell and reader task during
					// `localPty:createProcess`. `start` is a no-op that just
					// completes the workbench's launch promise. Returning
					// `Value::Null` matches `IPtyService.start`'s
					// `Promise<ITerminalLaunchError | ITerminalLaunchResult |
					// undefined>` (`undefined` branch). Routing this back
					// through `TerminalCreate` would spawn a SECOND
					// PTY for the same workbench terminal - the user-visible
					// pane is bound to id=1 from `createProcess`, but a
					// shadow PTY (id=2) starts and streams data nobody
					// renders.
					dev_log!("terminal", "{} no-op (eager-spawn)", command);
					Ok(Value::Null)
				},
				"localPty:input" | "localPty:write" => call!(rt, "terminal", TerminalSendText, Arguments),
				"localPty:shutdown" | "localPty:dispose" => call!(rt, "terminal", TerminalDispose, Arguments),
				"localPty:resize" => call!(rt, "terminal", "localPty:resize", LocalPTYResize, Arguments),
				"localPty:acknowledgeDataEvent" => {
					// xterm flow-control heartbeat; no-op on Mountain side.
					Ok(Value::Null)
				},
				// `ILocalPtyService.getBackendOS` - VS Code uses this to decide
				// which profile list to show (Windows/Linux/macOS). Returns the
				// `OperatingSystem` enum value from
				// `vs/base/common/platform.ts`: 1 = Macintosh, 2 = Linux, 3 = Windows.
				"localPty:getBackendOS" => {
					#[cfg(target_os = "macos")]
					{
						Ok(json!(1))
					}
					#[cfg(target_os = "linux")]
					{
						Ok(json!(2))
					}
					#[cfg(target_os = "windows")]
					{
						Ok(json!(3))
					}
					#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
					{
						Ok(json!(2))
					}
				},

				// `ILocalPtyService.refreshProperty` - returns the current value
				// of a PTY property. VS Code calls this for `ProcessId` (to show
				// PID in the terminal tab tooltip) and `Cwd` (for smart basename).
				// Property enum: 0=Cwd, 1=ProcessId, 2=Title, 3=OverrideName,
				// 4=ResolvedShellLaunchConfig, 5=ShellType
				// `ILocalPtyService.refreshProperty` - returns the current value
				// of a PTY property. VS Code calls this for `ProcessId` (tooltip)
				// and `Cwd` (smart basename).
				// Property enum: 0=Cwd, 1=ProcessId, 2=Title…
				"localPty:refreshProperty" => {
					use CommonLibrary::{
						Environment::Requires::Requires,
						Terminal::TerminalProvider::TerminalProvider,
					};
					let TerminalId = arg_u64(&Arguments, 0);
					let PropId = arg_u64(&Arguments, 1);
					if TerminalId == 0 {
						Ok(Value::Null)
					} else if PropId == 1 {
						let Provider:Arc<dyn TerminalProvider> = RunTime.Environment.Require();
						match Provider.GetTerminalProcessId(TerminalId).await {
							Ok(Some(Pid)) => Ok(json!(Pid)),
							_ => Ok(Value::Null),
						}
					} else {
						Ok(Value::Null)
					}
				},

				// `ILocalPtyService.updateProperty` - workbench sets icon/title
				// on a running PTY; acknowledged, no Mountain-side state change.
				"localPty:updateProperty" => Ok(Value::Null),

				// `ILocalPtyService.freePortKillProcess` - kill whatever process
				// is listening on a port so a new terminal can bind it.
				"localPty:freePortKillProcess" => {
					dev_log!("terminal", "localPty:freePortKillProcess");
					LocalPTYFreePortKillProcess(Arguments).await
				},

				// `ILocalPtyService.serializeTerminalProcesses` - snapshot all
				// active terminals so the workbench can persist them to storage
				// and restore them across a window reload. Returns
				// `ISerializedTerminalState[]`.
				"localPty:serializeTerminalState" => {
					dev_log!("terminal", "localPty:serializeTerminalState");
					SerializeTerminalState(RunTime.clone()).await
				},

				// `ILocalPtyService.reviveTerminalProcesses` - respawn shells from
				// a snapshot produced by `serializeTerminalState`. Accepts
				// `(ISerializedTerminalState[], dateTimeFormatLocale)`.
				"localPty:reviveTerminalProcesses" => {
					dev_log!(
						"terminal",
						"localPty:reviveTerminalProcesses count={}",
						Arguments.first().and_then(|V| V.as_array()).map(|A| A.len()).unwrap_or(0)
					);
					ReviveTerminalProcesses(RunTime.clone(), Arguments).await
				},

				// `ILocalPtyService.getRevivedPtyNewId` - allocate a fresh
				// terminal ID for a revived PTY. The workbench calls this before
				// `reviveTerminalProcesses` to pre-assign an integer it can use
				// to key into `_ptys`. Returning the next atomic counter value
				// keeps IDs unique and collision-free across reloads.
				"localPty:getRevivedPtyNewId" => {
					let NewId = RunTime.Environment.ApplicationState.GetNextTerminalIdentifier();
					dev_log!("terminal", "localPty:getRevivedPtyNewId id={}", NewId);
					Ok(json!(NewId))
				},

				// Session reconnect: reattach the workbench to a live Mountain
				// PTY after a window reload. The provider looks up the terminal
				// by id and returns its PID. DetachFromProcess is the inverse -
				// Mountain keeps the PTY running; output buffer accumulates for
				// the next attach or sky:replay-events drain.
				"localPty:attachToProcess" => {
					dev_log!("terminal", "localPty:attachToProcess");
					AttachToProcess(RunTime.clone(), Arguments).await
				},
				"localPty:detachFromProcess" => {
					dev_log!("terminal", "localPty:detachFromProcess");
					DetachFromProcess(RunTime.clone(), Arguments).await
				},

				// Remaining `localPty:*` - no Mountain-side state needed.
				// `installAutoReply` / `uninstallAllAutoReplies`: shell-integration
				// auto-reply triggers (e.g. sudo password prompts) - not implemented.
				"localPty:processBinary"
				| "localPty:orphanQuestionReply"
				| "localPty:updateTitle"
				| "localPty:updateIcon"
				| "localPty:installAutoReply"
				| "localPty:uninstallAllAutoReplies" => Ok(Value::Null),

				// =====================================================================
				// Update service - all stubs, no update server
				// =====================================================================
				"update:_getInitialState" => UpdateGetInitialState().await,
				"update:isLatestVersion" => UpdateIsLatestVersion().await,
				"update:checkForUpdates" => UpdateCheckForUpdates().await,
				"update:downloadUpdate" => UpdateDownloadUpdate().await,
				"update:applyUpdate" => UpdateApplyUpdate().await,
				"update:quitAndInstall" => UpdateQuitAndInstall().await,

				// =====================================================================
				// Menubar
				// =====================================================================
				// VS Code fires `updateMenubar` on every active-editor / dirty /
				// selection change - now handled in the high-frequency fast-path
				// (see the `if IsHighFrequencyCommand` block above). This fallback
				// only fires if the command somehow bypasses the fast-path.

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
				"encryption:encrypt" => Encrypt(Arguments).await,
				"encryption:decrypt" => Decrypt(Arguments).await,

				// =====================================================================
				// Extension host starter - atomic handlers
				// =====================================================================
				"extensionHostStarter:createExtensionHost" => {
					dev_log!("exthost", "extensionHostStarter:createExtensionHost");
					ExtensionHostStarterCreate(Arguments).await
				},
				"extensionHostStarter:start" => {
					dev_log!("exthost", "extensionHostStarter:start");
					ExtensionHostStarterStart(Arguments).await
				},
				"extensionHostStarter:kill" => {
					dev_log!("exthost", "extensionHostStarter:kill");
					ExtensionHostStarterKill(Arguments).await
				},
				"extensionHostStarter:getExitInfo" => {
					dev_log!("exthost", "extensionHostStarter:getExitInfo");
					ExtensionHostStarterGetExitInfo(Arguments).await
				},
				"extensionHostStarter:waitForExit" => {
					dev_log!("exthost", "extensionHostStarter:waitForExit");
					ExtensionHostStarterWaitForExit(Arguments).await
				},

				// =====================================================================
				// Extension host message relay (Wind → Mountain → Cocoon) - atomic
				// =====================================================================
				"cocoon:extensionHostMessage" => {
					dev_log!("exthost", "cocoon:extensionHostMessage");
					CocoonExtensionHostMessage(ApplicationHandle.clone(), Arguments).await
				},

				// =====================================================================
				// Extension host debug service - atomic handlers
				// =====================================================================
				"extensionhostdebugservice:reload" => {
					dev_log!("exthost", "extensionhostdebugservice:reload");
					ExtensionHostDebugReload(ApplicationHandle.clone()).await
				},
				"extensionhostdebugservice:close" => {
					dev_log!("exthost", "extensionhostdebugservice:close");
					ExtensionHostDebugClose(ApplicationHandle.clone()).await
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
					let Uri = arg_string(&Arguments, 0);
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
					let Entries:Vec<Value> = Arguments.first().and_then(|V| V.as_array()).cloned().unwrap_or_default();
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
					let Workspace = &RunTime.Environment.ApplicationState.Workspace;
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
					Git::HandleExec::Fn(Arguments).await
				},
				"git:clone" => {
					dev_log!("git", "git:clone");
					Git::HandleClone::Fn(Arguments).await
				},
				"git:pull" => {
					dev_log!("git", "git:pull");
					Git::HandlePull::Fn(Arguments).await
				},
				"git:checkout" => {
					dev_log!("git", "git:checkout");
					Git::HandleCheckout::Fn(Arguments).await
				},
				"git:revParse" => {
					dev_log!("git", "git:revParse");
					Git::HandleRevParse::Fn(Arguments).await
				},
				"git:fetch" => {
					dev_log!("git", "git:fetch");
					Git::HandleFetch::Fn(Arguments).await
				},
				"git:revListCount" => {
					dev_log!("git", "git:revListCount");
					Git::HandleRevListCount::Fn(Arguments).await
				},
				"git:cancel" => {
					dev_log!("git", "git:cancel");
					Git::HandleCancel::Fn(Arguments).await
				},
				"git:isAvailable" => {
					dev_log!("git", "git:isAvailable");
					Git::HandleIsAvailable::Fn(Arguments).await
				},

				// Tree-view child lookup from the renderer side. Mirrors the
				// Cocoon→Mountain `GetTreeChildren` gRPC path (see
				// `RPC/CocoonService/TreeView.rs::GetTreeChildren`) but is
				// invoked by the Wind/Sky tree-view bridge so the UI can
				// request children directly without waiting for Cocoon to
				// ask first. Delegated to atomic handler.
				"tree:getChildren" => TreeGetChildren(ApplicationHandle.clone(), RunTime.clone(), Arguments).await,

				// `treeView.reveal(element)` - focus/expand a specific item in the tree.
				// Emits a Sky event that triggers `IViewsService.openView(viewId)`.
				"tree.reveal" | "tree:reveal" => {
					use tauri::Emitter;
					let ViewId = arg_string(&Arguments, 0);
					let Handle = arg_string(&Arguments, 1);
					let Options = arg_val(&Arguments, 2);
					dev_log!("ipc", "tree.reveal viewId={} handle={}", ViewId, Handle);
					let _ = ApplicationHandle.emit(
						"sky://tree-view/reveal",
						json!({
							"viewId": ViewId,
							"handle": Handle,
							"options": Options,
						}),
					);
					Ok(Value::Null)
				},

				// Tree view UI interaction events forwarded from Sky → Mountain → Cocoon.
				// Sky emits these when the VS Code workbench fires treeView.onDidChangeSelection,
				// onDidCollapseElement, onDidExpandElement, onDidChangeVisibility.
				"tree:selectionChanged" | "tree:collapseElement" | "tree:expandElement" | "tree:visibilityChanged" => {
					let Payload = arg_val(&Arguments, 0);
					let Method = match command.as_str() {
						"tree:selectionChanged" => "$treeView:selectionChanged",
						"tree:collapseElement" => "$treeView:collapseElement",
						"tree:expandElement" => "$treeView:expandElement",
						_ => "$treeView:visibilityChanged",
					};
					tokio::spawn(async move {
						if let Err(E) = crate::Vine::Client::SendNotification::Fn(
							"cocoon-main".to_string(),
							Method.to_string(),
							Payload,
						)
						.await
						{
							dev_log!("ipc", "warn: [tree] Cocoon notify {} failed: {:?}", Method, E);
						}
					});
					Ok(Value::Null)
				},

				// SkyBridge event replay - delegated to atomic handler.
				"sky:replay-events" => SkyReplayEvents(ApplicationHandle.clone(), RunTime.clone()).await,

				// `editor.revealRange` - sky-side shortcut to scroll Monaco to a range.
				// Extensions can also call this via `Context.SendToMountain` (gRPC Track
				// Effect path). This IPC arm lets Wind call it directly without gRPC.
				"editor:revealRange" | "window:revealRange" => {
					use tauri::Emitter;
					let Payload = arg_val(&Arguments, 0);
					let _ = ApplicationHandle.emit("sky://editor/revealRange", &Payload);
					Ok(Value::Null)
				},

				// =====================================================================
				// Sky → Mountain editor state pushes
				// =====================================================================

				// Sky pushes current selection whenever the user changes cursor position.
				// Mountain stores it and forwards to Cocoon so `activeTextEditor.selection`
				// and `onDidChangeTextEditorSelection` stay live.
				"sky:editor:selectionChanged" => {
					let Uri = Arguments
						.first()
						.and_then(|V| V.get("uri"))
						.and_then(|V| V.as_str())
						.unwrap_or("")
						.to_string();
					let Selections = Arguments
						.first()
						.and_then(|V| V.get("selections"))
						.cloned()
						.unwrap_or(Value::Array(Vec::new()));
					dev_log!("model", "[SelectionChanged] uri={}", Uri);
					// Store on workspace state
					if !Uri.is_empty() {
						RunTime
							.Environment
							.ApplicationState
							.Workspace
							.SetActiveDocumentURI(Some(Uri.clone()));
					}
					let ViewColumn = Arguments
						.first()
						.and_then(|V| V.get("viewColumn"))
						.and_then(|V| V.as_u64())
						.unwrap_or(1);
					// Forward to Cocoon - include viewColumn so extensions
					// calling `activeTextEditor.viewColumn` see the correct
					// pane number in split-editor layouts.
					let Payload = json!({ "uri": Uri, "selections": Selections, "viewColumn": ViewColumn });
					let _ = crate::Vine::Client::SendNotification::Fn(
						"cocoon-main".to_string(),
						"window.didChangeTextEditorSelection".to_string(),
						Payload,
					)
					.await;
					Ok(Value::Null)
				},

				// Sky pushes active editor info when user switches tabs.
				// Sky sends model content changes (debounced) so Cocoon's
				// DocumentContentCache stays in sync with what the user is typing.
				// This enables LSP-backed diagnostics, completions, hover to see
				// up-to-date content without waiting for a file save.
				"sky:model:contentChanged" => {
					let Payload = arg_val(&Arguments, 0);
					let Uri = Payload.get("uri").and_then(Value::as_str).unwrap_or("").to_string();
					if !Uri.is_empty() {
						let Content = Payload.get("content").and_then(Value::as_str).unwrap_or("").to_string();
						let Version = Payload.get("version").and_then(Value::as_i64).unwrap_or(1);
						// Update in-memory document state.
						if let Some(mut Doc) = RunTime.Environment.ApplicationState.Feature.Documents.Get(&Uri) {
							Doc.Version = Version;
							Doc.Lines = Content.lines().map(|L| L.to_owned()).collect();
							Doc.IsDirty = true;
							RunTime
								.Environment
								.ApplicationState
								.Feature
								.Documents
								.AddOrUpdate(Uri.clone(), Doc);
						}
						// Notify Cocoon so onDidChangeTextDocument fires in extensions.
						let Payload2 = json!([
							{ "external": Uri.clone(), "$mid": 1 },

							{ "content": Content, "versionId": Version, "isDirty": true, "changes": [] }
						]);
						tokio::spawn(async move {
							let _ = crate::Vine::Client::SendNotification::Fn(
								"cocoon-main".to_string(),
								"$acceptModelChanged".to_string(),
								Payload2,
							)
							.await;
						});
					}
					Ok(Value::Null)
				},

				"sky:editor:activeChanged" => {
					let Payload = arg_val(&Arguments, 0);
					let Uri = Payload.get("uri").and_then(Value::as_str).unwrap_or("").to_string();
					dev_log!("model", "[ActiveEditorChanged] uri={}", Uri);
					if !Uri.is_empty() {
						RunTime
							.Environment
							.ApplicationState
							.Workspace
							.SetActiveDocumentURI(Some(Uri.clone()));
					}
					let _ = crate::Vine::Client::SendNotification::Fn(
						"cocoon-main".to_string(),
						"window.didChangeActiveTextEditor".to_string(),
						Payload,
					)
					.await;
					Ok(Value::Null)
				},

				// =====================================================================
				// Language features (forward to Cocoon Node.js runtime)
				// =====================================================================
				// These are VS Code language-intelligence channels. Mountain has no
				// native implementation - Cocoon's extension host processes them via
				// the LanguageProviderRegistry. All go through cocoon:request bridge.
				// Sky Bridge inline completion request: Sky's Monaco InlineCompletionsProvider
				// calls this when the editor requests ghost text for a cursor position.
				// Uses the public LanguageFeatureProviderRegistry trait to call the same
				// pipeline as Mountain's own gRPC ProvideInlineCompletionItems handler.
				"language:provideInlineCompletions" => {
					let Payload = arg_val(&Arguments, 0);
					let UriStr = Payload.get("uri").and_then(Value::as_str).unwrap_or("").to_string();

					if UriStr.is_empty() {
						Ok(json!({ "items": [] }))
					} else {
						let Line = Payload
							.get("position")
							.and_then(|P| P.get("line"))
							.and_then(Value::as_u64)
							.unwrap_or(0) as i64 + 1;
						let Character = Payload
							.get("position")
							.and_then(|P| P.get("character"))
							.and_then(Value::as_u64)
							.unwrap_or(0) as i64 + 1;
						let Context = Payload.get("context").cloned().unwrap_or_else(|| json!({ "triggerKind": 0 }));

						match url::Url::parse(&UriStr) {
							Ok(Uri) => {
								let Position = PositionDTO { LineNumber:Line as u32, Column:Character as u32 };
								match RunTime.Environment.ProvideInlineCompletionItems(Uri, Position, Context).await {
									Ok(Some(Result)) => {
										let Items = Result
											.get("items")
											.cloned()
											.unwrap_or_else(|| if Result.is_array() { Result } else { json!([]) });
										Ok(json!({ "items": Items }))
									},
									Ok(None) => Ok(json!({ "items": [] })),
									Err(Error) => {
										dev_log!("ipc", "warn: language:provideInlineCompletions error: {}", Error);
										Ok(json!({ "items": [] }))
									},
								}
							},
							Err(_) => Ok(json!({ "items": [] })),
						}
					}
				},

				"languages:getAll" | "languages:getEncodedLanguageId" => {
					dev_log!("extensions", "languages: {} (→ Cocoon)", command);
					let Payload = Arguments.into_iter().next().unwrap_or(Value::Null);
					// Skip the 3-second blocking wait at boot. If Cocoon isn't
					// connected yet, return an empty fallback immediately so the
					// tokenizer doesn't stall the worker for up to 3 s on first
					// editor open. The workbench retries on the next keystroke.
					// NOTE: must be an if/else expression, not `return Ok(...)`.
					// A bare `return` inside this match arm exits the enclosing
					// async block (not just the arm), changing the block's inferred
					// return type from `()` to `Result<Value, _>` and breaking
					// Scheduler::Submit's Output = () bound.
					if !crate::Vine::Client::IsClientConnected::Fn("cocoon-main") {
						Ok(Value::Array(Vec::new()))
					} else {
						Ok(
							crate::Vine::Client::SendRequest::Fn("cocoon-main", command.clone(), Payload, 5_000)
								.await
								.unwrap_or(Value::Array(Vec::new())),
						)
					}
				},

				// =====================================================================
				// SCM - forward to Cocoon's vscode.scm namespace
				// =====================================================================
				"scm:createSourceControl" | "scm:getSourceControls" | "scm:setActiveProvider" => {
					forward_to_cocoon!("scm", command, Arguments)
				},

				// =====================================================================
				// Debug - forward to Cocoon's vscode.debug namespace
				// =====================================================================
				"debug:startDebugging"
				| "debug:stopDebugging"
				| "debug:getSessions"
				| "debug:getBreakpoints"
				| "debug:addBreakpoints"
				| "debug:removeBreakpoints" => {
					forward_to_cocoon!("debug", command, Arguments)
				},

				// =====================================================================
				// Tasks - forward to Cocoon's vscode.tasks namespace
				// =====================================================================
				"tasks:executeTask" | "tasks:getTasks" | "tasks:getTaskExecution" => {
					forward_to_cocoon!("tasks", command, Arguments)
				},

				// =====================================================================
				// Authentication - forward to Cocoon's vscode.authentication namespace
				// =====================================================================
				"auth:getSessions" | "auth:createSession" | "auth:removeSession" => {
					forward_to_cocoon!("auth", command, Arguments)
				},

				// Atom L2 + NodeDeferred: unknown-command fallback.
				// First consults the Channel registry (three states):
				//   1. typo / never-registered → log + defer to Cocoon
				//   2. registered but no dispatch arm → log + defer to Cocoon
				//   3. Cocoon returns error → surface as IPC error
				//
				// When `TierIPC=NodeDeferred` or `TierIPC=Node` (set in
				// .env.Land) unknown commands are forwarded to Cocoon's
				// Node.js runtime via gRPC instead of returning an error.
				// This lets VS Code API surfaces that live in the extension
				// host (language features, SCM, debug, tasks, etc.) resolve
				// without requiring a Mountain dispatch arm.
				_ => {
					use std::str::FromStr;

					// Check if command should defer to Cocoon's Node.js runtime.
					// The env var is baked in at build time via rustc-env from
					// build.rs; at runtime we also accept it via process env for
					// debug overrides.
					let TierIPC = std::env::var("TierIPC").unwrap_or_else(|_| "Mountain".into());
					let ShouldDefer = TierIPC == "NodeDeferred" || TierIPC == "Node";

					if ShouldDefer {
						// Forward to Cocoon via cocoon:request bridge.
						// Cocoon's RequestRoutingHandler + extension namespaces
						// cover language:*, scm:*, debug:*, tasks:*, auth:*, etc.
						let Payload = cocoon_payload(Arguments);
						dev_log!("ipc", "deferred → Cocoon: {}", command);
						let _ = crate::Vine::Client::WaitForClientConnection::Fn("cocoon-main", 3000).await;
						match crate::Vine::Client::SendRequest::Fn("cocoon-main", command.clone(), Payload, 15_000)
							.await
						{
							Ok(Response) => Ok(Response),
							Err(CocoonError) => {
								dev_log!(
									"ipc",
									"warn: [NodeDeferred] {} deferred but Cocoon rejected: {:?}",
									command,
									CocoonError
								);
								Ok(Value::Null)
							},
						}
					} else {
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
	// Skip for high-frequency silenced calls to avoid thousands of spans
	// per session (logger, file I/O, storage polling).
	if !IsHighFrequencyCommand {
		let IsErr = Result.is_err();

		let SpanName = if IsErr {
			format!("land:mountain:ipc:{}:error", command)
		} else {
			format!("land:mountain:ipc:{}", command)
		};

		crate::otel_span!(&SpanName, OTLPStart, &[("ipc.command", command.as_str())]);

		// Emit `land:mountain:handler:complete` to PostHog for every dispatched IPC.
		// Pairs with `land:cocoon:handler:complete` to populate the Feature
		// Parity dashboard's Node-vs-Rust handler-latency comparison.
		let HandlerElapsedNanos = crate::IPC::DevLog::NowNano::Fn().saturating_sub(OTLPStart);

		let HandlerDurationMs = HandlerElapsedNanos / 1_000_000;

		crate::Binary::Build::PostHogPlugin::CaptureHandler::Fn(&command, HandlerDurationMs, !IsErr);
	}

	// Atom I13: paired entry/exit line per invoke. `invoke: <cmd>` on the way
	// in (emitted at the top of this fn); `done: <cmd> ok=… t_ns=…` on the
	// way out. A `grep "logger:log"` before showed only the entry half;
	// having both halves makes latency diagnosis a single pipe:
	//     grep "logger:log" Mountain.dev.log | awk '…'
	// without hopping across Jaeger. High-frequency commands still skip the
	// entry line but DO emit an exit - frequencies still aggregate, but each
	// is individually accounted for.
	if !IsHighFrequencyCommand {
		let ElapsedNanos = crate::IPC::DevLog::NowNano::Fn().saturating_sub(OTLPStart);

		dev_log!("ipc", "done: {} ok={} t_ns={}", command, !Result.is_err(), ElapsedNanos);
	}

	Result
}

pub fn register_wind_ipc_handlers(ApplicationHandle:&tauri::AppHandle) -> Result<(), String> {
	dev_log!("lifecycle", "registering IPC handlers");

	// Note: These handlers are automatically registered when included in the
	// Tauri invoke_handler macro in the main binary

	Ok(())
}
