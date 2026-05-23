#![allow(non_snake_case)]
//! UI-layer IPC handlers - one `pub async fn Fn` per file.

pub mod DecorationsClear;

pub mod DecorationsGet;

pub mod DecorationsGetMany;

pub mod DecorationsSet;

pub mod KeybindingAdd;

pub mod KeybindingGetAll;

pub mod KeybindingLookup;

pub mod KeybindingRemove;

pub mod LifecycleGetPhase;

pub mod LifecycleRequestShutdown;

pub mod LifecycleWhenPhase;

pub mod NotificationEndProgress;

pub mod NotificationShow;

pub mod NotificationShowProgress;

pub mod NotificationUpdateProgress;

pub mod ProgressBegin;

pub mod ProgressEnd;

pub mod ProgressReport;

pub mod QuickInputShowInputBox;

pub mod QuickInputShowQuickPick;

pub mod ThemesGetActive;

pub mod ThemesList;

pub mod ThemesSet;

pub mod WorkingCopyGetAllDirty;

pub mod WorkingCopyGetDirtyCount;

pub mod WorkingCopyIsDirty;

pub mod WorkingCopySetDirty;

pub mod WorkspacesAddFolder;

pub mod WorkspacesGetFolders;

pub mod WorkspacesGetName;

pub mod WorkspacesRemoveFolder;
