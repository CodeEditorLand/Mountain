#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! NativeHost atoms - native OS-layer handlers.
//!
//! One `pub async fn` per file. This `mod.rs` only declares sub-modules; no
//! `pub use`. Dispatch-site callers bring each atom in via local `use`.

pub mod Exit;

pub mod FindFreePort;

pub mod GetColorScheme;

pub mod InstallShellCommand;

pub mod IsFullscreen;

pub mod IsMaximized;

pub mod KillProcess;

pub mod OpenDevTools;

pub mod OpenExternal;

pub mod OSProperties;

pub mod OSStatistics;

pub mod PickFolder;

pub mod Quit;

pub mod Reload;

pub mod Relaunch;

pub mod ShowItemInFolder;

pub mod ShowOpenDialog;

pub mod ToggleDevTools;

pub mod UninstallShellCommand;

pub mod Clipboard;

pub mod GetEnvironmentPaths;

pub mod IsRunningUnderARM64Translation;

pub mod MoveItemToTrash;

pub mod ShowMessageBox;

pub mod ShowSaveDialog;
