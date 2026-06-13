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

pub mod CloseWindow;

pub mod Exit;

pub mod FindFreePort;

pub mod FocusWindow;

pub mod GetColorScheme;

pub mod GetEnvironmentPaths;

pub mod GetSystemIdleState;

pub mod GetSystemIdleTime;

pub mod GetWebSocketConfig;

pub mod GetWindows;

pub mod HasWSLFeatureInstalled;

pub mod InstallShellCommand;

pub mod IsFullscreen;

pub mod IsMaximized;

pub mod IsPortFree;

pub mod IsRunningUnderARM64Translation;

pub mod KillProcess;

pub mod MaximizeWindow;

pub mod MinimizeWindow;

pub mod MoveItemToTrash;

pub mod OnDidChangeMaximizeState;

pub mod OpenDevTools;

pub mod OpenExternal;

pub mod OSProperties;

pub mod OSStatistics;

pub mod PickFolder;

pub mod PositionWindow;

pub mod Quit;

pub mod Relaunch;

pub mod Reload;

pub mod ResolveProxy;

pub mod Router;

pub mod SetDocumentEdited;

pub mod SetMinimumSize;

pub mod SetRepresentedFilename;

pub mod SetTitle;

pub mod SetWindowAlwaysOnTop;

pub mod ShowItemInFolder;

pub mod ShowMessageBox;

pub mod ShowOpenDialog;

pub mod ShowSaveDialog;

pub mod ShowSaveDialogUI;

pub mod StartPowerSaveBlocker;

pub mod ToggleDevTools;

pub mod ToggleFullScreen;

pub mod ToggleWindowAlwaysOnTop;

pub mod UninstallShellCommand;

pub mod UnmaximizeWindow;
