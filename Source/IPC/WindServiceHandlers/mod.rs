//! Wind Service Handlers - dispatcher and sub-module aggregator.
//! Domain files handle the individual handler implementations.
pub mod register_wind_ipc_handlers;
pub mod mountain_ipc_invoke;

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
	ApplicationRoot::{Get::Fn as GetStaticApplicationRoot, Set::Fn as SetStaticApplicationRoot},
	ChannelPriority::Fn as ResolveChannelPriority,
	FiddeeRoot::Fn as FiddeeRoot,
	JsonValueHelpers::{
		ArgBool,
		ArgBoolTrue,
		ArgF64,
		ArgI64,
		ArgStr,
		ArgString,
		ArgStringOr,
		ArgU64,
		ArgU64Or,
		ArgVal,
		Fn as VStr,
		ReqString,
	},
	MetadataEncoding::Fn as MetadataToIStat,
	PathExtraction::Fn as ExtractPathFromArg,
	PercentDecode::Fn as PercentDecode,
	RecentlyOpened::{
		Mutate::Fn as MutateRecentlyOpened,
		Path::Fn as RecentlyOpenedPath,
		Read::Fn as ReadRecentlyOpened,
	},
	UserdataDir::{Ensure::Fn as EnsureUserdataDirs, Get::Fn as GetUserdataBaseDir, Set::Fn as SetUserdataBaseDir},
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
		Struct::{
			ApplicationState::ApplicationState,
			WorkspaceState::WorkspaceDelta::UpdateWorkspaceFoldersAndBroadcast,
		},
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

fn CocoonPayload(Args:Vec<Value>) -> Value {
	match Args.len() {
		0 => Value::Null,
		1 => Args.into_iter().next().unwrap(),
		_ => Value::Array(Args),
	}
}

macro_rules! forward_to_cocoon {
	($tag:literal, $command:ident, $Arguments:ident) => {{
		dev_log!("ipc", "{}: {} (→ Cocoon)", $tag, $command);
		let Payload = CocoonPayload($Arguments);
		let _ = crate::Vine::Client::WaitForClientConnection::Fn("cocoon-main", 3000).await;
		Ok(
			crate::Vine::Client::SendRequest::Fn("cocoon-main", $command.clone(), Payload, 10_000)
				.await
				.unwrap_or(Value::Null),
		)
	}};
}

#[derive(Debug, Clone)]
pub struct Struct;
