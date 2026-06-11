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

pub mod Workspaces;

pub mod Dispatcher;

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
		FileReadFd::Fn as FileReadFd,
		FileReadNative::Fn as FileReadNative,
		FileReaddirNative::Fn as FileReaddirNative,
		FileRealpath::Fn as FileRealpath,
		FileRenameNative::Fn as FileRenameNative,
		FileStatNative::Fn as FileStatNative,
		FileUnwatch::Fn as FileUnwatch,
		FileWatch::Fn as FileWatch,
		FileWriteFd::Fn as FileWriteFd,
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
	OutputDispose::Fn as OutputDispose,
	OutputReplace::Fn as OutputReplace,
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
	KeybindingEvaluateWhen::Fn as KeybindingEvaluateWhen,
	KeybindingGetAll::Fn as KeybindingGetAll,
	KeybindingLookup::Fn as KeybindingLookup,
	KeybindingRemove::Fn as KeybindingRemove,
	KeybindingResolve::Fn as KeybindingResolve,
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
use tauri::{AppHandle, Emitter, Manager};
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
	Configuration::{ConfigurationInspector::ConfigurationInspector, ConfigurationProvider::ConfigurationProvider},
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
// Compile-time tier baselines baked by build.rs::EmitTierDefaults. Each
// dispatch arm reads the matching const and routes Mountain-native vs.
// Cocoon Node.js per the `.env.Land` (or flavor overlay) value. When a
// dev override is needed without a rebuild, the `tier_runtime!` macro
// below picks up the process env var first so a single shell export
// (`export TierStorage=Node`) flips routing immediately.
const TIER_TERMINAL:&str = env!("TierTerminal", "Mountain");

const TIER_SCM:&str = env!("TierSCM", "Mountain");

const TIER_DEBUG:&str = env!("TierDebug", "Mountain");

const TIER_LANGUAGE_FEATURES:&str = env!("TierLanguageFeatures", "Mountain");

const TIER_SEARCH:&str = env!("TierSearch", "Mountain");

const TIER_OUTPUT_CHANNEL:&str = env!("TierOutputChannel", "Mountain");

const TIER_NATIVE_HOST:&str = env!("TierNativeHost", "Mountain");

const TIER_TREE_VIEW:&str = env!("TierTreeView", "Mountain");

const TIER_STORAGE:&str = env!("TierStorage", "Mountain");

const TIER_MODEL:&str = env!("TierModel", "Mountain");

const TIER_TASKS:&str = env!("TierTasks", "Node");

const TIER_AUTH:&str = env!("TierAuth", "Node");

const TIER_ENCRYPTION:&str = env!("TierEncryption", "Mountain");

const TIER_WEBSOCKET:&str = env!("TierWebSocket", "Disabled");

#[inline]
fn tier_routes_to_node(BakedConst:&'static str, EnvKey:&str) -> bool {
	let Resolved = std::env::var(EnvKey).unwrap_or_else(|_| BakedConst.to_string());

	Resolved == "Node"
}

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
			| "storage:keys"
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
			// Storage lifecycle stubs (storage:optimize excluded - it must flush pending writes)
			| "storage:isUsed" | "storage:close"
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

			// Defined here (not at module level) so macro hygiene resolves
			// `RunTime`, `ApplicationHandle`, and `command` from this scope.
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

				(app, $tag:literal, $msg:literal, $Fn:path) => {{
					dev_log!($tag, $msg);

					$Fn(ApplicationHandle.clone()).await
				}};
			}

			let MatchResult = match command.as_str() {
				// Configuration commands. VS Code's stock
				// `ConfigurationService` channel calls `getValue` /
				// `updateValue`; Mountain's native Effect-TS layer calls
				// `get` / `update`. Alias both to the same handler so
				// traffic from either rail lands in the same place.
				"configuration:get" | "configuration:getValue" => call!(rt, "config", ConfigurationGet, Arguments),
				"configuration:update" | "configuration:updateValue" => {
					let UpdateResult = call!(rt, "config", ConfigurationUpdate, Arguments);

					// On successful update, broadcast the change to Sky so
					// the workbench theme/settings UI reflects the new value
					// without a full reload.
					if UpdateResult.is_ok() {
						let _ = ApplicationHandle.emit("sky://configuration/changed", serde_json::json!({}));
					}

					UpdateResult
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
				// VS Code destructures `{ value, defaultValue, userValue,
				// workspaceValue, workspaceFolderValue, memoryValue }` from
				// the result. `InspectConfigurationValue` reads each scope
				// individually (default extensions, user settings.json,
				// workspace settings.json) so the Settings UI can show which
				// scope is overriding a given key.
				"configuration:inspect" => {
					dev_log!("config", "configuration:inspect");

					let Key = arg_string(&Arguments, 0);

					let Inspector:Arc<dyn ConfigurationInspector> = RunTime.Environment.Require();

					match Inspector
						.InspectConfigurationValue(Key.clone(), ConfigurationOverridesDTO::default())
						.await
					{
						Ok(Some(Result)) => {
							Ok(json!({
								"value": Result.EffectiveValue,
								"defaultValue": Result.DefaultValue,
								"userValue": Result.UserValue,
								"workspaceValue": Result.WorkspaceValue,
								"workspaceFolderValue": Result.WorkspaceFolderValue,
								"memoryValue": Result.MemoryValue,
							}))
						},

						Ok(None) => {
							// Key not found in any scope - fall back to merged value
							// so the Settings UI gets `undefined` rather than crashing.
							let Fallback = ConfigurationGet(RunTime.clone(), Arguments).await.unwrap_or(Value::Null);

							Ok(json!({
								"value": Fallback,
								"defaultValue": Fallback,
								"userValue": Value::Null,
								"workspaceValue": Value::Null,
								"workspaceFolderValue": Value::Null,
								"memoryValue": Value::Null,
							}))
						},

						Err(Error) => {
							dev_log!("config", "warn: configuration:inspect error for '{}': {}", Key, Error);

							Ok(json!({
								"value": Value::Null,
								"defaultValue": Value::Null,
								"userValue": Value::Null,
								"workspaceValue": Value::Null,
								"workspaceFolderValue": Value::Null,
								"memoryValue": Value::Null,
							}))
						},
					}
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
				| "logger:setVisibility" => Ok(Value::Null),

				// Return an empty array so the VS Code Output view can
				// iterate the result without crashing on null.
				"logger:getRegisteredLoggers" => Ok(json!([])),

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
				// `read`/`write` are OVERLOADED: the fd-based stream path
				// (`DiskFileSystemProviderClient.read/write` - behind every
				// editor file open via `readFileStream` and every streamed
				// save) sends a NUMERIC fd as the first argument, while the
				// legacy whole-file form sends a URI/path. Dispatch on the
				// argument shape; routing fd calls into FileReadNative made
				// PathExtraction reject the number and every editor open
				// land in `workbench.editors.errorEditor`.
				"file:read" if Arguments.first().is_some_and(Value::is_number) => FileReadFd(Arguments).await,
				"file:read" | "file:readFile" => FileReadNative(Arguments).await,
				"file:write" if Arguments.first().is_some_and(Value::is_number) => FileWriteFd(Arguments).await,
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
				//
				// TierStorage gate: `TierStorage=Node` forwards every
				// storage:* call to Cocoon (vscode.ExtensionContext's
				// globalState/workspaceState lives there too). Default
				// is Mountain - ApplicationState in-memory map with the
				// MementoLoader crash-safe boot hydration and the
				// debounced disk writer.
				"storage:get"
				| "storage:set"
				| "storage:getItems"
				| "storage:updateItems"
				| "storage:optimize"
				| "storage:isUsed"
				| "storage:close"
				| "storage:delete"
				| "storage:keys"
				| "storage:onDidChangeItems"
				| "storage:logStorage"
					if tier_routes_to_node(TIER_STORAGE, "TierStorage") =>
				{
					forward_to_cocoon!("storage", command, Arguments)
				},
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
					// Flush pending debounced writes for both scopes immediately.
					// VS Code calls this before workspace close and hot-reload to
					// ensure state is fully persisted without waiting for the 100 ms
					// debounce window. The call carries no scope argument, so global
					// and workspace stores flush together - a partial flush could
					// persist one scope while losing in-flight writes to the other.
					dev_log!("storage", "storage:optimize → flush");

					let GlobalPath = Some((*RunTime.Environment.ApplicationState.GlobalMementoPath.lock()).clone());

					let WorkspacePath = (*RunTime.Environment.ApplicationState.WorkspaceMementoPath.lock()).clone();

					let GlobalData =
						(*RunTime.Environment.ApplicationState.Configuration.MementoGlobalStorage.lock()).clone();

					let WorkspaceData = (*RunTime
						.Environment
						.ApplicationState
						.Configuration
						.MementoWorkspaceStorage
						.lock())
					.clone();

					crate::Environment::StorageProvider::FlushPendingWrites(
						GlobalPath,
						WorkspacePath,
						GlobalData,
						WorkspaceData,
					)
					.await;

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

				// `env:asExternalUri` / `env:resolveExternalUri` - VS Code resolves
				// extension-provided URIs through these before opening them externally
				// or embedding them in webviews. Without Mountain registering an
				// opener, the URI passes through unchanged.
				"env:asExternalUri" | "env:resolveExternalUri" => {
					let UriStr = arg_string(&Arguments, 0);

					Ok(serde_json::json!({ "uri": UriStr }))
				},

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
				"commands:getAll" | "commands:getCommands" => call!(rt, "commands", CommandsGetAll),
				// Register/unregister from a side-car channel perspective
				// is a no-op: Cocoon sends `$registerCommand` via gRPC
				// (handled elsewhere). Ack Null so the workbench side
				// doesn't hang on a promise.
				"commands:registerCommand"
				| "commands:unregisterCommand"
				| "commands:onDidRegisterCommand"
				| "commands:onDidExecuteCommand" => Ok(Value::Null),

				// Extension host commands
				"extensions:getAll" => call!(rt, "extensions", "extensions:getAll", ExtensionsGetAll),
				"extensions:get" => call!(rt, "extensions", "extensions:get", ExtensionsGet, Arguments),
				"extensions:isActive" => call!(rt, "extensions", "extensions:isActive", ExtensionsIsActive, Arguments),
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
				// empty arrays / properly-shaped envelopes for every read, which
				// mirrors a network-air-gapped VS Code session. Each shape must
				// match VS Code's `IGalleryQueryResult` exactly so the Extensions
				// view renders "0 results" instead of crashing with a type error.
				"extensions:query" | "extensions:getExtensions" | "extensions:getRecommendations" => {
					dev_log!("extensions", "{} (offline gallery - returning [])", command);

					Ok(Value::Array(Vec::new()))
				},

				// `ExtensionGalleryService.query()` - called when the user types
				// in the Extensions search box. Returns `IGalleryQueryResult`:
				// `{ galleryExtensions: IExtension[], total: number }`. An empty
				// envelope stops the "loading…" spinner and shows "0 results".
				"extensions:search" => {
					dev_log!("extensions", "extensions:search (offline gallery - returning empty)");

					Ok(json!({ "galleryExtensions": [], "total": 0 }))
				},

				// `ExtensionGalleryService.getCoreTranslation()` - locale bundles.
				// Returns null so VS Code falls back to the bundled English strings.
				"extensions:getCoreTranslation" => {
					Ok(Value::Null)
				},

				// `ExtensionGalleryService.download()` - called when installing a
				// marketplace extension. With no gallery backend the download
				// always fails. Return an error shape VS Code surfaces to the user
				// as "marketplace unavailable" rather than a JS TypeError.
				"extensions:download" => {
					Err("Marketplace download unavailable in offline mode".to_string())
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
					crate::IPC::WindServiceHandlers::Extension::ExtensionGetManifest::Fn(Arguments).await
				},
				// `extensions:reinstall` - returns a minimal ILocalExtension envelope
				// so VS Code's ExtensionManagementService doesn't retry the operation.
				// No gallery backend is available; the on-disk unpack is unchanged.
				"extensions:reinstall" => {
					let ExtId = arg_string(&Arguments, 0);

					dev_log!("extensions", "extensions:reinstall {} (no-op: no gallery)", ExtId);

					Ok(serde_json::json!({ "identifier": { "id": ExtId }, "version": "0.0.0", "type": 0 }))
				},

				// Metadata update only matters for ratings/icons/readme which Land
				// does not track. Left as explicit log so the UI doesn't silently fail.
				"extensions:updateMetadata" => {
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
				"output:replace" => call!(app, "output", "output:replace", OutputReplace, Arguments),
				"output:dispose" => call!(app, "output", "output:dispose", OutputDispose, Arguments),

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
				// Abort an in-flight text-search task by search_id.
				"search:cancel" => {
					if let Some(SearchId) = Arguments.first().and_then(|V| V.as_u64()) {
						// Cooperative flag first: the synchronous ripgrep
						// walk polls it per entry, which is what actually
						// stops the CPU work. The task abort below only
						// lands at an await point.
						let Flags = &RunTime.Environment.ApplicationState.Feature.SearchCancellationFlags;

						if let Some(Flag) = Flags.get(&SearchId) {
							Flag.store(true, std::sync::atomic::Ordering::Relaxed);
						}

						let ActiveSearches = &RunTime.Environment.ApplicationState.Feature.ActiveSearches;

						if let Some((_, Handle)) = ActiveSearches.remove(&SearchId) {
							Handle.abort();

							dev_log!("search", "search:cancel aborted id={}", SearchId);
						} else {
							dev_log!("search", "search:cancel id={} not found (already done?)", SearchId);
						}
					} else {
						dev_log!("search", "search:cancel (no id, ignoring)");
					}

					Ok(Value::Null)
				},

				// No-op acks for search channel methods that have no
				// server-side state.
				"search:clearCache" | "search:onDidChangeResult" => {
					dev_log!("search", "{} (stub-ack)", command);

					Ok(Value::Null)
				},

				// Decorations commands
				"decorations:get" => call!(rt, "decorations", "decorations:get", DecorationsGet, Arguments),
				"decorations:getMany" => call!(rt, "decorations", "decorations:getMany", DecorationsGetMany, Arguments),
				"decorations:set" => call!(rt, "decorations", "decorations:set", DecorationsSet, Arguments),
				"decorations:clear" => call!(rt, "decorations", "decorations:clear", DecorationsClear, Arguments),

				// WorkingCopy commands
				"workingCopy:isDirty" => call!(rt, "workingcopy", "workingCopy:isDirty", WorkingCopyIsDirty, Arguments),
				"workingCopy:setDirty" => {
					call!(rt, "workingcopy", "workingCopy:setDirty", WorkingCopySetDirty, Arguments)
				},
				"workingCopy:getAllDirty" => {
					call!(rt, "workingcopy", "workingCopy:getAllDirty", WorkingCopyGetAllDirty)
				},
				"workingCopy:getDirtyCount" => {
					call!(rt, "workingcopy", "workingCopy:getDirtyCount", WorkingCopyGetDirtyCount)
				},

				// Keybinding commands
				"keybinding:add" => call!(rt, "keybinding", "keybinding:add", KeybindingAdd, Arguments),
				"keybinding:remove" => call!(rt, "keybinding", "keybinding:remove", KeybindingRemove, Arguments),
				"keybinding:lookup" => call!(rt, "keybinding", "keybinding:lookup", KeybindingLookup, Arguments),
				"keybinding:getAll" => call!(rt, "keybinding", "keybinding:getAll", KeybindingGetAll),
				"keybinding:resolve" => {
					call!(rt, "keybinding", "keybinding:resolve", KeybindingResolve, Arguments)
				},
				"keybinding:evaluateWhen" => {
					call!(rt, "keybinding", "keybinding:evaluateWhen", KeybindingEvaluateWhen, Arguments)
				},

				// Lifecycle commands
				"lifecycle:getPhase" => call!(rt, "lifecycle", "lifecycle:getPhase", LifecycleGetPhase),
				"lifecycle:whenPhase" => call!(rt, "lifecycle", "lifecycle:whenPhase", LifecycleWhenPhase, Arguments),
				"lifecycle:requestShutdown" => {
					call!(app, "lifecycle", "lifecycle:requestShutdown", LifecycleRequestShutdown)
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

				// `workspace.openTextDocument(uri)` - Sky-side Cocoon call that
				// asks Mountain to relay the open intent to Sky so Monaco loads
				// the document and emits `onDidOpenTextDocument`.
				"text:open" | "workspace:openTextDocument" => {
					let UriStr = arg_string(&Arguments, 0);

					if !UriStr.is_empty() {
						// A failed emit is intentionally ignored: the open intent is
						// fire-and-forget, and Sky may not be listening yet during
						// boot. The caller treats Null as "request relayed".
						let _ = ApplicationHandle
							.emit("sky://window/showTextDocument", serde_json::json!({ "uri": UriStr }));
					}

					Ok(Value::Null)
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
					let FolderCount = RunTime.Environment.ApplicationState.Workspace.WorkspaceFolders.lock().len();

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

				// B7-S6: WebSocket config for direct Sky<->Cocoon transport.
				"nativeHost:getWebSocketConfig" => {
					use crate::ProcessManagement::CocoonManagement::{WsPort, WsSecretHex};

					Ok(serde_json::json!({ "port": WsPort(), "secret": WsSecretHex() }))
				},

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

				// VS Code's `IWindowsMainService.getWindowCount()` equivalent
				// for the renderer side. Land runs a single window; returning
				// 1 is accurate and avoids the "last window" guard in VS Code's
				// lifecycle service from suppressing quit prompts.
				"window:getActiveWindowCount" => Ok(json!(1)),

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

				// `window:setTitle` / `nativeHost:setTitle` - explicit title
				// override from extensions or the workbench title service.
				"window:setTitle" | "nativeHost:setTitle" => {
					dev_log!("window", "{}", command);

					let Title = arg_string(&Arguments, 0);

					if !Title.is_empty() {
						if let Some(Win) = ApplicationHandle.get_webview_window("main") {
							let _ = Win.set_title(&Title);
						}
					}

					Ok(Value::Null)
				},

				// `NSWindow.isDocumentEdited` - the ● dirty dot in the macOS title
				// bar. Tauri 2.x does not expose NSWindow::setDocumentEdited
				// directly; prefix the window title with '•' as a visual proxy.
				"nativeHost:setDocumentEdited" => {
					let Edited = Arguments.first().and_then(Value::as_bool).unwrap_or(false);

					if let Some(Win) = ApplicationHandle.get_webview_window("main") {
						if let Ok(Current) = Win.title() {
							let New = if Edited {
								if Current.starts_with('•') { Current } else { format!("• {}", Current) }
							} else {
								Current.trim_start_matches('•').trim().to_string()
							};

							let _ = Win.set_title(&New);
						}
					}

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
				// Return empty credentials so the proxy layer's JSON parse succeeds.
				"nativeHost:lookupAuthorization" => Ok(json!({"username":"","password":""})),
				// Contract is `Promise<string | undefined>` returning the raw
				// token string (native.ts lookupKerberosAuthorization); null
				// deserialises to undefined = "no authorization available",
				// the safe answer with no Kerberos backend.
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
				// updateKeybindings/updateTheme/updateConfiguration forward the
				// payload to Sky so webview overlays receive live config updates.
				// openDevTools/closeDevTools are no-ops (devtools are native).
				"browserView:updateKeybindings" => {
					use tauri::Emitter;

					let Payload = arg_val(&Arguments, 0);

					let _ = ApplicationHandle.emit("sky://webview/keybindings-update", &Payload);

					Ok(Value::Null)
				},

				"browserView:updateTheme" => {
					use tauri::Emitter;

					let Payload = arg_val(&Arguments, 0);

					let _ = ApplicationHandle.emit("sky://webview/theme-update", &Payload);

					Ok(Value::Null)
				},

				"browserView:updateConfiguration" => {
					use tauri::Emitter;

					let Payload = arg_val(&Arguments, 0);

					let _ = ApplicationHandle.emit("sky://webview/configuration-update", &Payload);

					Ok(Value::Null)
				},

				"browserView:openDevTools" | "browserView:closeDevTools" => Ok(Value::Null),
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

					// `IPtyService.listProcesses` returns `IProcessDetails[]`
					// (`vs/platform/terminal/common/terminal.ts`). The
					// workbench uses it for terminal-tab tooltips and the
					// reconnect-on-reload flow. Build entries from the live
					// PTY registry; `isOrphan:false` because Mountain spawns
					// PTYs in-process (they die with us, so there is never a
					// detached pty-host process to revive from).
					let Terminals = RunTime.Environment.ApplicationState.Feature.Terminals.GetAll();

					let mut Entries:Vec<_> = Terminals.values().collect();

					Entries.sort_by_key(|T| T.Identifier);

					let Processes:Vec<Value> = Entries
						.into_iter()
						.map(|T| {
							json!({
								"id": T.Identifier,
								"title": if T.Title.is_empty() { T.Name.clone() } else { T.Title.clone() },
								"titleSource": 0,
								"pid": T.OSProcessIdentifier.unwrap_or(0),
								"cwd": T.GetWorkingDirectory(),
								"workspaceId": "",
								"workspaceName": "",
								"isOrphan": false,
								"icon": Value::Null,
								"color": Value::Null,
								"fixedDimensions": Value::Null,
								"environmentVariableCollections": Value::Null,
								"shellLaunchConfig": {
									"executable": T.ShellPath,
									"args": T.ShellArguments,
								},
								"hasChildProcesses": false,
								"type": Value::Null,
								"hideFromUser": false,
								"isFeatureTerminal": false,
							})
						})
						.collect();

					Ok(json!(Processes))
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
				"localPty:refreshProperty" => {
					use CommonLibrary::{
						Environment::Requires::Requires,
						Terminal::TerminalProvider::TerminalProvider,
					};

					let TerminalId = arg_u64(&Arguments, 0);

					let PropId = arg_u64(&Arguments, 1);

					if TerminalId == 0 {
						Ok(Value::Null)
					} else if PropId == 0 {
						// TerminalProperty::Cwd - return last OSC 633 P;cwd= value
						let Cwd = {
							let guard = RunTime.Environment.ApplicationState.Feature.Terminals.ActiveTerminals.lock();

							guard
								.get(&TerminalId)
								.cloned()
								.and_then(|S| {
									let s_guard = S.lock();

									s_guard.CurrentWorkingDirectory.clone()
								})
								.map(|P| P.to_string_lossy().into_owned())
						};

						Ok(Cwd.map(|C| json!(C)).unwrap_or(Value::Null))
					} else if PropId == 1 {
						// TerminalProperty::ProcessId
						let Provider:Arc<dyn TerminalProvider> = RunTime.Environment.Require();

						match Provider.GetTerminalProcessId(TerminalId).await {
							Ok(Some(Pid)) => Ok(json!(Pid)),
							_ => Ok(Value::Null),
						}
					} else {
						Ok(Value::Null)
					}
				},

				// `ILocalPtyService.updateProperty` - workbench notifies Mountain
				// of a property change on a running PTY. Property enum:
				//   2 = Title (dynamic title from shell escape)
				//   3 = OverrideName (user-renamed tab)
				//   5 = ShellType (detected shell identifier)
				// Title / OverrideName are persisted in TerminalStateDTO and
				// forwarded to Sky so the xterm tab label updates live.
				// ShellType is stored for later `refreshProperty` lookups.
				"localPty:updateProperty" => {
					use CommonLibrary::IPC::SkyEvent::SkyEvent;

					let TermId = arg_u64(&Arguments, 0);

					let PropId = arg_u64(&Arguments, 1);

					let PropValue = Arguments.get(2).and_then(Value::as_str).unwrap_or("").to_string();

					if TermId == 0 || PropValue.is_empty() {
						Ok(Value::Null)
					} else {
						match PropId {
							// Title (2) or OverrideName (3): persist + emit to Sky.
							2 | 3 => {
								{
									let Guard =
										RunTime.Environment.ApplicationState.Feature.Terminals.ActiveTerminals.lock();

									if let Some(Entry) = Guard.get(&TermId) {
										Entry.lock().Title = PropValue.clone();
									}
								}

								dev_log!(
									"terminal",
									"localPty:updateProperty id={} prop={} title='{}'",
									TermId,
									PropId,
									PropValue
								);

								let _ = RunTime.Environment.ApplicationHandle.emit(
									SkyEvent::TerminalPropertyChanged.AsStr(),
									json!({
										"id": TermId,
										"property": PropId,
										"value": PropValue,
									}),
								);
							},

							// ShellType (5): store only; workbench derives its own icon.
							5 => {
								{
									let Guard =
										RunTime.Environment.ApplicationState.Feature.Terminals.ActiveTerminals.lock();

									if let Some(Entry) = Guard.get(&TermId) {
										Entry.lock().ShellType = Some(PropValue.clone());
									}
								}

								dev_log!(
									"terminal",
									"localPty:updateProperty id={} shell_type='{}'",
									TermId,
									PropValue
								);
							},

							Other => {
								dev_log!(
									"terminal",
									"localPty:updateProperty id={} unknown_prop={} (no-op)",
									TermId,
									Other
								);
							},
						}

						Ok(Value::Null)
					} // closes else
				},

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

				// `ILocalPtyService.getRevivedPtyNewId` - return the new terminal
				// ID assigned to an old (pre-reload) ID during
				// `reviveTerminalProcesses`. Arguments: `[workspaceId, oldId]`.
				// The mapping is populated by `ReviveTerminalProcesses` and
				// consumed here on first lookup. Falls back to a fresh ID so a
				// missing entry never hangs the workbench.
				"localPty:getRevivedPtyNewId" => {
					let OldId = arg_u64(&Arguments, 1);

					let MaybeNewId = if OldId != 0 {
						RunTime
							.Environment
							.ApplicationState
							.Feature
							.Terminals
							.RevivedIdMap
							.lock()
							.remove(&OldId)
					} else {
						None
					};

					let NewId =
						MaybeNewId.unwrap_or_else(|| RunTime.Environment.ApplicationState.GetNextTerminalIdentifier());

					dev_log!("terminal", "localPty:getRevivedPtyNewId old_id={} new_id={}", OldId, NewId);

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

				// `localPty:setActive` - fired by Sky Bridge when the user
				// switches terminal tabs. Notifies Cocoon so that
				// `vscode.window.activeTerminal` reflects the focused terminal.
				"localPty:setActive" => {
					let TermId = Arguments.first().and_then(Value::as_i64);

					let Payload = match TermId {
						Some(Id) => serde_json::json!({ "id": Id }),
						None => serde_json::json!({ "id": null }),
					};

					let _ = crate::Vine::Client::SendNotification::Fn(
						"cocoon-main".to_string(),
						"$acceptActiveTerminalChanged".to_string(),
						Payload,
					)
					.await;

					Ok(Value::Null)
				},

				// `localPty:setShellIntegrationActive` - Sky fires once per
				// terminal when OSC 633 ; A (prompt start) is first detected.
				// Notifies Cocoon so `terminal.shellIntegration !== undefined`
				// and `onDidChangeTerminalShellIntegration` fires.
				"localPty:setShellIntegrationActive" => {
					let TermId = Arguments.first().and_then(Value::as_i64).unwrap_or(0) as u64;

					let _ = crate::Vine::Client::SendNotification::Fn(
						"cocoon-main".to_string(),
						"$acceptTerminalShellIntegrationActivated".to_string(),
						serde_json::json!({ "id": TermId }),
					)
					.await;

					Ok(Value::Null)
				},

				// `localPty:setInteracted` - Sky fires once per terminal when
				// it detects OSC 633 ; B (command-input-begins). Forwards to
				// Cocoon as `$acceptTerminalStateChanged` so subscribers of
				// `vscode.window.onDidChangeTerminalState` see
				// `state.isInteractedWith` flip true. Payload from Sky:
				// `[{ id, interactedWith }]`.
				"localPty:setInteracted" => {
					let Payload = arg_val(&Arguments, 0);

					let _ = crate::Vine::Client::SendNotification::Fn(
						"cocoon-main".to_string(),
						"$acceptTerminalStateChanged".to_string(),
						Payload,
					)
					.await;

					Ok(Value::Null)
				},

				// `localPty:setCwd` - Sky Bridge fires this when it parses an
				// OSC 633 P;cwd=<path> sequence from terminal output. Mountain
				// forwards to Cocoon so `vscode.window.activeTerminal.
				// shellIntegration.cwd` reflects the shell's current directory.
				"localPty:setCwd" => {
					let TermId = Arguments.first().and_then(Value::as_i64).unwrap_or(0) as u64;

					let Cwd = Arguments.get(1).and_then(Value::as_str).unwrap_or("").to_string();

					if !Cwd.is_empty() {
						// Persist CWD in ApplicationState. Lock, update, drop immediately.
						let _CwdPersisted = RunTime
							.Environment
							.ApplicationState
							.Feature
							.Terminals
							.ActiveTerminals
							.lock()
							.get(&TermId)
							.map(|E| {
								E.lock().CurrentWorkingDirectory = Some(std::path::PathBuf::from(&Cwd));
							})
							.is_some();

						let _ = crate::Vine::Client::SendNotification::Fn(
							"cocoon-main".to_string(),
							"$acceptTerminalCwdChange".to_string(),
							serde_json::json!({ "id": TermId, "cwd": Cwd }),
						)
						.await;
					}

					Ok(Value::Null)
				},

				// `localPty:processBinary` - the workbench forwards raw binary
				// input (paste of UTF-16, Ctrl+sequences from xterm.js) here
				// instead of through `input`. Previously dropped to null, which
				// meant pasting from system clipboard or sending Cmd+Shift
				// keyboard escape sequences was silently swallowed. Route
				// through the same TerminalSendText path the input channel
				// uses so the bytes reach the PTY.
				"localPty:processBinary" => {
					call!(rt, "terminal", TerminalSendText, Arguments)
				},
				// Remaining `localPty:*` - no Mountain-side state needed.
				"localPty:orphanQuestionReply" | "localPty:updateTitle" | "localPty:updateIcon" => Ok(Value::Null),

				// `ILocalPtyService.installAutoReply` - store an auto-reply rule
				// so the PTY reader can respond automatically to matching output
				// (e.g. password prompts, Y/N confirmations).
				// Payload: `{ answer, match, useCustomAnswer }`.
				"localPty:installAutoReply" => {
					use crate::ApplicationState::State::FeatureState::Terminals::TerminalState::AutoReplyRule;

					let Payload = arg_val(&Arguments, 0);

					let Answer = Payload.get("answer").and_then(Value::as_str).unwrap_or("").to_string();

					let MatchStr = Payload.get("match").and_then(Value::as_str).unwrap_or("").to_string();

					let UseCustom = Payload.get("useCustomAnswer").and_then(Value::as_bool).unwrap_or(false);

					if !Answer.is_empty() && !MatchStr.is_empty() {
						RunTime
							.Environment
							.ApplicationState
							.Feature
							.Terminals
							.AutoReplies
							.lock()
							.push(AutoReplyRule {
								Match:MatchStr.clone(),
								Answer:Answer.clone(),
								UseCustomAnswer:UseCustom,
							});

						dev_log!("terminal", "localPty:installAutoReply match='{}' answer='{}'", MatchStr, Answer);
					}

					Ok(Value::Null)
				},

				// `ILocalPtyService.uninstallAllAutoReplies` - clear every
				// installed auto-reply rule for the current session.
				"localPty:uninstallAllAutoReplies" => {
					RunTime
						.Environment
						.ApplicationState
						.Feature
						.Terminals
						.AutoReplies
						.lock()
						.clear();

					dev_log!("terminal", "localPty:uninstallAllAutoReplies cleared");

					Ok(Value::Null)
				},

				// `localPty:shellExecutionStart` - Sky fires this when it
				// detects OSC 633 ; C (command-output-begins) in terminal
				// data. Payload: `{ id, commandLine, cwd }`. Forward to
				// Cocoon so `vscode.window.onDidStartTerminalShellExecution`
				// subscribers see the execution event. The subscriber lives
				// at `Window/Namespace.ts` on the
				// `window.didStartTerminalShellExecution` Emitter channel.
				"localPty:shellExecutionStart" => {
					let Payload = arg_val(&Arguments, 0);

					let _ = crate::Vine::Client::SendNotification::Fn(
						"cocoon-main".to_string(),
						"$acceptTerminalShellExecutionStart".to_string(),
						Payload,
					)
					.await;

					Ok(Value::Null)
				},

				// `localPty:shellExecutionEnd` - Sky fires this when it
				// detects OSC 633 ; D (command-finished) in terminal data.
				// Payload: `{ id, commandLine, cwd, exitCode }`. Fans to
				// Cocoon as both `$acceptTerminalShellExecutionEnd` (for
				// `onDidEndTerminalShellExecution`) AND a derived
				// `$acceptExecutedTerminalCommand` so
				// `vscode.window.onDidExecuteTerminalCommand` subscribers
				// see the executed command without a separate Sky-side
				// detection pass (the shape is a subset of the end
				// event - same data, different consumer audience).
				"localPty:shellExecutionEnd" => {
					let Payload = arg_val(&Arguments, 0);

					let _ = crate::Vine::Client::SendNotification::Fn(
						"cocoon-main".to_string(),
						"$acceptTerminalShellExecutionEnd".to_string(),
						Payload.clone(),
					)
					.await;

					let _ = crate::Vine::Client::SendNotification::Fn(
						"cocoon-main".to_string(),
						"$acceptExecutedTerminalCommand".to_string(),
						Payload,
					)
					.await;

					Ok(Value::Null)
				},

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
					// Arguments shape (ExtHostUrls.registerExternalUriOpener):
					//   [0] opener_id: String
					//   [1] schemes: String | String[]
					//   [2] extension_id: String
					let OpenerId = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_owned();

					let ExtensionId = Arguments.get(2).and_then(|V| V.as_str()).unwrap_or("").to_owned();

					let Schemes:Vec<String> = match Arguments.get(1) {
						Some(Value::Array(Arr)) => Arr.iter().filter_map(|V| V.as_str().map(str::to_owned)).collect(),
						Some(Value::String(S)) => vec![S.clone()],
						_ => vec![],
					};

					dev_log!(
						"url",
						"url:registerExternalUriOpener: id={} ext={} schemes={:?}",
						OpenerId,
						ExtensionId,
						Schemes
					);

					{
						use crate::ApplicationState::State::FeatureState::State::ExternalUriOpenerRegistration;

						let mut Guard = RunTime.Environment.ApplicationState.Feature.ExternalUriOpeners.lock();

						for Scheme in Schemes {
							Guard.insert(
								Scheme.clone(),
								ExternalUriOpenerRegistration {
									Scheme,
									ExtensionId:ExtensionId.clone(),
									OpenerId:OpenerId.clone(),
								},
							);
						}
					}

					Ok(Value::Null)
				},

				// =====================================================================
				// Encryption
				// =====================================================================
				"encryption:encrypt" => Encrypt(Arguments).await,
				"encryption:decrypt" => Decrypt(Arguments).await,

				// =====================================================================
				// Process introspection - Wind queries platform/arch/pid/memory
				// =====================================================================
				// VS Code's shared-process service calls these for diagnostics and
				// for the "About" dialog. Most values are also in ISandboxConfiguration
				// but Wind may request them independently after boot.
				"process:getPlatform" => {
					Ok(json!(match std::env::consts::OS {
						"windows" => "win32",
						"macos" => "darwin",
						_ => "linux",
					}))
				},

				"process:getArch" => {
					Ok(json!(match std::env::consts::ARCH {
						"x86_64" => "x64",
						"aarch64" => "arm64",
						"x86" => "ia32",
						_ => "x64",
					}))
				},

				"process:getPid" => Ok(json!(std::process::id())),

				"process:getExecPath" => {
					Ok(json!(
						std::env::current_exe().unwrap_or_default().to_string_lossy().into_owned()
					))
				},

				"process:getMemoryInfo" => {
					// Electron's `process.getProcessMemoryInfo()` shape, in
					// KILOBYTES (VS Code's About/diagnostics panel multiplies
					// accordingly). Real resident-set numbers via sysinfo;
					// peak and private/shared splits aren't exposed by
					// sysinfo, so resident is reported for all three
					// resident-derived fields - close enough for the
					// diagnostics display this feeds.
					let ResidentKB = sysinfo::get_current_pid()
						.ok()
						.and_then(|Pid| {
							let mut System = sysinfo::System::new();

							System.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[Pid]), true);

							System.process(Pid).map(|Process| Process.memory() / 1024)
						})
						.unwrap_or(0);

					Ok(json!({
						"workingSetSize": ResidentKB,
						"peakWorkingSetSize": ResidentKB,
						"privateBytes": ResidentKB,
						"sharedBytes": 0u64,
					}))
				},

				"process:getCpuInfo" => {
					// Return a single-entry array matching Node.js `os.cpus()` shape.
					Ok(json!([{
						"model": format!("{} ({})", std::env::consts::ARCH, std::env::consts::OS),
						"speed": 0u32,
						"times": { "user": 0u64, "nice": 0u64, "sys": 0u64, "idle": 0u64, "irq": 0u64 },
					}]))
				},

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
				// `attachSession`: Wind signals that the extension-host debug
				// side-channel is ready for session `sessionId` on `port`.
				// Record a lightweight session entry in DebugState so
				// Mountain-side DAP routing can find the sidecar.
				"extensionhostdebugservice:attachSession" => {
					let SessionId = arg_string(&Arguments, 0);

					let Port = arg_u64(&Arguments, 1);

					let SubId = arg_string_or(&Arguments, 2, "cocoon-main");

					dev_log!(
						"exthost",
						"extensionhostdebugservice:attachSession id={} port={} sub={}",
						SessionId,
						Port,
						SubId
					);

					if !SessionId.is_empty() {
						let AlreadyRegistered = RunTime
							.Environment
							.ApplicationState
							.Feature
							.Debug
							.GetDebugSession(&SessionId)
							.is_some();

						if !AlreadyRegistered {
							let _ = RunTime.Environment.ApplicationState.Feature.Debug.RegisterDebugSession(
								crate::ApplicationState::State::FeatureState::Debug::DebugState::DebugSessionEntry {
									SessionId:SessionId.clone(),
									DebugType:"unknown".to_string(),
									SideCarIdentifier:SubId,
									StdinSender:None,
									ChildPid:None,
								},
							);
						}
					}

					Ok(Value::Null)
				},

				// `terminateSession`: Wind signals session end. Remove from
				// DebugState so stale entries don't accumulate.
				"extensionhostdebugservice:terminateSession" => {
					let SessionId = arg_string(&Arguments, 0);

					dev_log!("exthost", "extensionhostdebugservice:terminateSession id={}", SessionId);

					if !SessionId.is_empty() {
						RunTime
							.Environment
							.ApplicationState
							.Feature
							.Debug
							.UnregisterDebugSession(&SessionId);
					}

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
					Workspaces::EnterWorkspace::Fn(ApplicationHandle.clone(), RunTime.clone(), Arguments).await
				},
				"workspaces:createUntitledWorkspace" => {
					Workspaces::CreateUntitledWorkspace::Fn(ApplicationHandle.clone()).await
				},
				"workspaces:deleteUntitledWorkspace" => {
					Workspaces::DeleteUntitledWorkspace::Fn(ApplicationHandle.clone(), Arguments).await
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
				// Returns the display name for the current workspace.
				// VS Code's `IWorkspaceContextService.getWorkspaceName()` calls
				// this to populate the window title and the Explorer header.
				// Delegates to the same LabelGetWorkspace logic used by
				// `label:getWorkspace` so the name is consistent across callers.
				"workspaces:getWorkspaceName" => LabelGetWorkspace(RunTime.clone()).await,

				// `IWorkspacesService.getDirtyWorkspaces` lists OTHER windows'
				// workspaces that left hot-exit backups on disk, so a fresh
				// window can offer to restore them. Mountain has no backup
				// (hot-exit) service - in-window dirty state lives in
				// `Feature::WorkingCopy` and is queried via
				// `workingCopy:getAllDirty` - so the empty list is the
				// correct "no backups found" answer, not a stub.
				"workspaces:getDirtyWorkspaces" => Ok(json!([])),

				// Git (localGit channel) - implements stock VS Code's
				// ILocalGitService surface plus `exec` / `isAvailable` for
				// the built-in Git extension. Handlers spawn native `git`
				// via tokio::process. See Batch 4 in HANDOFF §-10.
				//
				// TierSCM gate: with `TierSCM=Node` (set in `.env.Land`
				// or via a flavor overlay) every git:* + scm:* command
				// forwards to Cocoon's vscode.scm namespace instead so
				// extensions like the upstream Git extension can run
				// pure-JS against their own bundled `simple-git`. Default
				// is Mountain - native subprocess with 30s timeout.
				"git:exec" | "git:clone" | "git:pull" | "git:checkout" | "git:revParse" | "git:fetch"
				| "git:revListCount" | "git:cancel" | "git:isAvailable"
					if tier_routes_to_node(TIER_SCM, "TierSCM") =>
				{
					forward_to_cocoon!("scm", command, Arguments)
				},
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

					// Mirror visibility state into ApplicationState so native
					// queries (sky:replay-events, treeView.visible getter) read
					// the current value without a Cocoon round-trip.
					if command == "tree:visibilityChanged" {
						if let (Some(ViewId), Some(Visible)) = (
							Payload.get("viewId").and_then(|v| v.as_str()),
							Payload.get("visible").and_then(|v| v.as_bool()),
						) {
							RunTime
								.Environment
								.ApplicationState
								.Feature
								.TreeViews
								.SetVisible(ViewId, Visible);
						}
					}

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

					// Fire onLanguage:<id> activation event for the newly
					// active editor so extensions that gate on the language
					// type (e.g. Rust Analyzer, ESLint, Prettier) activate
					// without needing an explicit `$activateByEvent` call from
					// the workbench. The languageId is looked up from the
					// in-memory document registry; unknown URIs are skipped.
					if !Uri.is_empty() {
						let MaybeLanguageId = RunTime
							.Environment
							.ApplicationState
							.Feature
							.Documents
							.Get(&Uri)
							.map(|Doc| Doc.LanguageIdentifier.clone());

						if let Some(LanguageId) = MaybeLanguageId {
							if !LanguageId.is_empty() && LanguageId != "plaintext" {
								dev_log!("extensions", "onLanguage:{} activation for uri={}", LanguageId, Uri);

								let _ = crate::Vine::Client::SendNotification::Fn(
									"cocoon-main".to_string(),
									"$activateByEvent".to_string(),
									json!({ "activationEvent": format!("onLanguage:{}", LanguageId) }),
								)
								.await;
							}
						}
					}

					Ok(Value::Null)
				},

				// Sky-detected visible-editors change. Forwarded by
				// `Bridge/InstallEditorOperations.ts` whenever
				// `IEditorService.onDidVisibleEditorsChange` fires. Payload:
				// `{ uris: string[] }` (the URIs of editors currently visible
				// in any group). Mountain fans to Cocoon as
				// `$acceptVisibleEditorsChanged` so
				// `vscode.window.onDidChangeVisibleTextEditors` subscribers
				// receive the change. Without this, linters that clear
				// diagnostics on close (rust-analyzer, ESLint) leave stale
				// markers when the user navigates between files.
				"sky:editor:visibleChanged" => {
					let Payload = arg_val(&Arguments, 0);

					let _ = crate::Vine::Client::SendNotification::Fn(
						"cocoon-main".to_string(),
						"$acceptVisibleEditorsChanged".to_string(),
						Payload,
					)
					.await;

					Ok(Value::Null)
				},

				// Sky-detected tab-model snapshot. Forwarded whenever any
				// `IEditorGroupsService` group's model mutates (open / close /
				// move / pin / split). Payload: `{ groups: [{ id, isActive,
				// tabs: [{ label, uri }] }] }`. Mountain fans the snapshot to
				// Cocoon as `$acceptTabsChanged`; Cocoon's NotificationHandler
				// re-emits on `window.didChangeTabs` AND `window.didChangeTabGroups`
				// (VS Code surfaces both events from the same underlying
				// group-model change). Used by tab-tracking extensions
				// (GitLens, Roo Code) and the `vscode.window.tabGroups` API.
				"sky:editor:tabsChanged" => {
					let Payload = arg_val(&Arguments, 0);

					let _ = crate::Vine::Client::SendNotification::Fn(
						"cocoon-main".to_string(),
						"$acceptTabsChanged".to_string(),
						Payload,
					)
					.await;

					Ok(Value::Null)
				},

				// Monaco scroll-driven visible-range change. Sky debounces
				// to ~60 ms before forwarding. Payload: `{ uri, viewColumn,
				// visibleRanges }`. Fans to Cocoon as
				// `$acceptVisibleRangesChanged` so
				// `vscode.window.onDidChangeTextEditorVisibleRanges`
				// subscribers (code lens, lazy-load gutter contributions)
				// see scroll changes without the workbench-level event
				// loop.
				"sky:editor:visibleRangesChanged" => {
					let Payload = arg_val(&Arguments, 0);

					let _ = crate::Vine::Client::SendNotification::Fn(
						"cocoon-main".to_string(),
						"$acceptVisibleRangesChanged".to_string(),
						Payload,
					)
					.await;

					Ok(Value::Null)
				},

				// Monaco config-driven editor-option change (tab size,
				// insert-spaces, word-wrap, line numbers, etc.). Sky
				// forwards the resolved option set. Fans to Cocoon as
				// `$acceptTextEditorOptionsChanged` so
				// `vscode.window.onDidChangeTextEditorOptions` subscribers
				// fire. Most extensions only care about tab-size /
				// insert-spaces; the full Monaco change-set is included
				// so future consumers can read whatever they need.
				"sky:editor:optionsChanged" => {
					let Payload = arg_val(&Arguments, 0);

					let _ = crate::Vine::Client::SendNotification::Fn(
						"cocoon-main".to_string(),
						"$acceptTextEditorOptionsChanged".to_string(),
						Payload,
					)
					.await;

					Ok(Value::Null)
				},

				// `sky:editor:diffInformationChanged` - Sky detects when the
				// active editor pane is a diff editor and Monaco's
				// `onDidUpdateDiff` fires. Payload:
				//   `{ modifiedUri, originalUri, changes: LineChange[] }`.
				// Fans to Cocoon as `$acceptTextEditorDiffInformationChanged`
				// so subscribers of
				// `vscode.window.onDidChangeTextEditorDiffInformation` fire.
				"sky:editor:diffInformationChanged" => {
					let Payload = arg_val(&Arguments, 0);

					let _ = crate::Vine::Client::SendNotification::Fn(
						"cocoon-main".to_string(),
						"$acceptTextEditorDiffInformationChanged".to_string(),
						Payload,
					)
					.await;

					Ok(Value::Null)
				},

				// `sky:editor:viewColumnChanged` - Sky detects when an editor
				// is moved between editor groups (split-view shuffle,
				// drag-and-drop tab, `View: Move Editor to Group`) via
				// per-group `onDidMoveEditor`. Payload: `{ uri, viewColumn }`
				// where viewColumn is 1-based. Fans to Cocoon as
				// `$acceptTextEditorViewColumnChanged` so subscribers of
				// `vscode.window.onDidChangeTextEditorViewColumn` fire.
				"sky:editor:viewColumnChanged" => {
					let Payload = arg_val(&Arguments, 0);

					let _ = crate::Vine::Client::SendNotification::Fn(
						"cocoon-main".to_string(),
						"$acceptTextEditorViewColumnChanged".to_string(),
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

				"language:getLanguages" | "languages:getLanguages" => {
					// Builtin baseline merged with every `contributes.languages`
					// id from the scanned extensions, so the language picker /
					// `vscode.languages.getLanguages()` reflects what's actually
					// installed instead of a frozen 14-entry list.
					let mut Languages:Vec<String> = [
						"plaintext",
						"typescript",
						"javascript",
						"rust",
						"python",
						"go",
						"java",
						"cpp",
						"c",
						"html",
						"css",
						"json",
						"yaml",
						"markdown",
					]
					.iter()
					.map(|S| (*S).to_string())
					.collect();

					let Extensions = RunTime
						.Environment
						.ApplicationState
						.Extension
						.ScannedExtensions
						.ScannedExtensions
						.lock()
						.clone();

					for Extension in Extensions.values() {
						if let Some(Contributed) = Extension
							.Contributes
							.as_ref()
							.and_then(|C| C.get("languages"))
							.and_then(Value::as_array)
						{
							for Entry in Contributed {
								if let Some(Id) = Entry.get("id").and_then(Value::as_str) {
									Languages.push(Id.to_string());
								}
							}
						}
					}

					Languages.sort();

					Languages.dedup();

					Ok(json!(Languages))
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
				// Call hierarchy - forward to Cocoon's LanguageProviderRegistry
				// =====================================================================
				// VS Code calls these when the user invokes "Show Call Hierarchy"
				// (Shift+Alt+H). The extension host registers providers via
				// `vscode.languages.registerCallHierarchyProvider`; Cocoon's
				// LanguageProviderRegistry routes each request to the correct
				// extension. Mountain's gRPC handlers exist but are thin shims;
				// the authoritative implementation lives in the extension host.
				"language:prepareCallHierarchy"
				| "language:provideCallHierarchyIncomingCalls"
				| "language:provideCallHierarchyOutgoingCalls" => {
					forward_to_cocoon!("language", command, Arguments)
				},

				// =====================================================================
				// Type hierarchy - forward to Cocoon's LanguageProviderRegistry
				// =====================================================================
				"language:prepareTypeHierarchy"
				| "language:provideTypeHierarchySupertypes"
				| "language:provideTypeHierarchySubtypes" => {
					forward_to_cocoon!("language", command, Arguments)
				},

				// =====================================================================
				// Linked editing ranges - forward to Cocoon
				// =====================================================================
				"language:provideLinkedEditingRanges" => {
					forward_to_cocoon!("language", command, Arguments)
				},

				// =====================================================================
				// SCM - Mountain-native reads; mutations forwarded to Cocoon
				// =====================================================================

				// Returns the serialized list of all registered SCM providers
				// directly from ApplicationState, avoiding a Cocoon round-trip.
				"scm:getSourceControls" => {
					let Providers:Vec<_> = RunTime
						.Environment
						.ApplicationState
						.Feature
						.Markers
						.SourceControlManagementProviders
						.lock()
						.values()
						.cloned()
						.collect();

					Ok(json!(Providers))
				},

				// Records the active provider handle in ApplicationState so
				// sky:replay-events and future Mountain-native SCM queries can
				// surface the current focus without a Cocoon round-trip.
				"scm:setActiveProvider" => {
					use std::sync::atomic::Ordering as AtomicOrdering;

					let Handle = arg_u64(&Arguments, 0) as u32;

					RunTime
						.Environment
						.ApplicationState
						.Feature
						.Markers
						.ActiveSourceControlManagementProvider
						.store(Handle, AtomicOrdering::Relaxed);

					dev_log!("scm", "scm:setActiveProvider handle={}", Handle);

					Ok(json!(null))
				},

				"scm:createSourceControl" => {
					forward_to_cocoon!("scm", command, Arguments)
				},

				// =====================================================================
				// Debug - Mountain-native pre-processing + Cocoon forward
				// =====================================================================

				// `debug:startDebugging`: call DebugProvider::StartDebugging
				// (via the Debug.Start effect) to register the session and
				// optionally spawn the DAP adapter before forwarding to
				// Cocoon so vscode.debug extension listeners see a live
				// session.
				"debug:startDebugging" => {
					let _ = TIER_DEBUG;

					let FolderUriStr = arg_string_or(&Arguments, 0, "");

					let Config = arg_val(&Arguments, 1);

					let DebugStartParams = json!([FolderUriStr, Config]);

					let StartEffect = crate::Track::Effect::CreateEffectForRequest::Debug::CreateEffect::<tauri::Wry>(
						"Debug.Start",
						DebugStartParams,
					);

					if let Some(EffectResult) = StartEffect {
						match EffectResult {
							Ok(task) => {
								if let Err(e) = task(RunTime.clone()).await {
									dev_log!("exthost", "warn: debug:startDebugging effect failed: {}", e);
								}
							},

							Err(e) => {
								dev_log!("exthost", "warn: debug:startDebugging effect build error: {}", e);
							},
						}
					}

					forward_to_cocoon!("debug", command, Arguments)
				},

				// `debug:addBreakpoints`: store in ApplicationState and emit
				// sky://debug/breakpointsChanged for renderer decorations,
				// then forward to Cocoon for onDidChangeBreakpoints.
				"debug:addBreakpoints" => {
					let _ = TIER_DEBUG;

					if let Some(serde_json::Value::Array(RawBreakpoints)) = Arguments.first() {
						let Entries:Vec<
							crate::ApplicationState::State::FeatureState::Debug::DebugState::BreakpointEntry,
						> = RawBreakpoints
							.iter()
							.filter_map(|Raw| {
								let Id = Raw
									.get("id")
									.or_else(|| Raw.get("Id"))
									.and_then(serde_json::Value::as_str)
									.map(str::to_string)?;

								let Kind = Raw
									.get("type")
									.or_else(|| Raw.get("kind"))
									.and_then(serde_json::Value::as_str)
									.unwrap_or("source")
									.to_string();

								let Uri = Raw
									.get("uri")
									.or_else(|| Raw.get("source").and_then(|S| S.get("uri")))
									.and_then(serde_json::Value::as_str)
									.unwrap_or("")
									.to_string();

								let Line = Raw
									.get("lineNumber")
									.or_else(|| Raw.get("line"))
									.and_then(serde_json::Value::as_u64)
									.unwrap_or(0);

								let Column = Raw.get("column").and_then(serde_json::Value::as_u64);

								let Enabled = Raw.get("enabled").and_then(serde_json::Value::as_bool).unwrap_or(true);

								Some(
									crate::ApplicationState::State::FeatureState::Debug::DebugState::BreakpointEntry {
										id:Id,
										kind:Kind,
										uri:Uri,
										line:Line,
										column:Column,
										enabled:Enabled,
										raw:Raw.clone(),
									},
								)
							})
							.collect();

						if !Entries.is_empty() {
							RunTime.Environment.ApplicationState.Feature.Debug.AddBreakpoints(Entries);

							let _ = ApplicationHandle.emit(
								"sky://debug/breakpointsChanged",
								json!({
									"added": RawBreakpoints,
									"removed": [],
									"changed": [],
								}),
							);
						}
					}

					forward_to_cocoon!("debug", command, Arguments)
				},

				// `debug:getBreakpoints`: served from Mountain's local store;
				// no Cocoon round-trip needed.
				"debug:getBreakpoints" => {
					let _ = TIER_DEBUG;

					let Bps = RunTime.Environment.ApplicationState.Feature.Debug.GetBreakpoints();

					Ok(json!(Bps))
				},

				// `debug:removeBreakpoints`: evict from local store and emit
				// change event, then forward to Cocoon.
				"debug:removeBreakpoints" => {
					let _ = TIER_DEBUG;

					if let Some(serde_json::Value::Array(RawIds)) = Arguments.first() {
						let Ids:Vec<String> = RawIds.iter().filter_map(|V| V.as_str().map(str::to_string)).collect();

						if !Ids.is_empty() {
							RunTime.Environment.ApplicationState.Feature.Debug.RemoveBreakpoints(&Ids);

							let _ = ApplicationHandle.emit(
								"sky://debug/breakpointsChanged",
								json!({
									"added": [],
									"removed": RawIds,
									"changed": [],
								}),
							);
						}
					}

					forward_to_cocoon!("debug", command, Arguments)
				},

				"debug:stopDebugging" | "debug:getSessions" => {
					let _ = TIER_DEBUG;

					forward_to_cocoon!("debug", command, Arguments)
				},

				// =====================================================================
				// Tasks - forward to Cocoon's vscode.tasks namespace
				// =====================================================================
				"tasks:executeTask" | "tasks:getTasks" => {
					forward_to_cocoon!("tasks", command, Arguments)
				},

				// Look up an active task execution by run-ID from the
				// in-process registry. Returns the stored definition JSON
				// or null if the ID is unknown (task already ended or never
				// started via Mountain's gRPC path).
				"tasks:getTaskExecution" => {
					let Id = arg_u64(&Arguments, 0);

					let Result = RunTime.Environment.ApplicationState.Feature.Tasks.Get(Id);

					Ok(Result.unwrap_or(Value::Null))
				},

				// =====================================================================
				// Authentication - forward to Cocoon's vscode.authentication namespace
				// =====================================================================
				"auth:getSessions" | "auth:createSession" | "auth:removeSession" | "auth:validateToken" => {
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
