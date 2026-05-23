#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! NativeHost atoms - native OS-layer handlers.
//!
//! One `pub async fn Fn` per file. This mod.rs only declares sub-modules.

pub mod ClipboardHas;

pub mod ClipboardReadBuffer;

pub mod ClipboardReadFindText;

pub mod ClipboardReadImage;

pub mod ClipboardReadText;

pub mod ClipboardTriggerPaste;

pub mod ClipboardWriteBuffer;

pub mod ClipboardWriteFindText;

pub mod ClipboardWriteText;

pub mod Exit;

pub mod FindFreePort;

pub mod GetColorScheme;

pub mod GetEnvironmentPaths;

pub mod InstallShellCommand;

pub mod IsFullscreen;

pub mod IsMaximized;

pub mod IsRunningUnderARM64Translation;

pub mod KillProcess;

pub mod MoveItemToTrash;

pub mod OpenDevTools;

pub mod OpenExternal;

pub mod OSProperties;

pub mod OSStatistics;

pub mod PickFolder;

pub mod Quit;

pub mod Reload;

pub mod Relaunch;

pub mod ShowItemInFolder;

pub mod ShowMessageBox;

pub mod ShowOpenDialog;

pub mod ShowSaveDialog;

pub mod ShowSaveDialogUI;

pub mod ToggleDevTools;

pub mod UninstallShellCommand;
