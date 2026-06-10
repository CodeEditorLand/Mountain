//! Domain dispatchers - each module handles a group of related IPC commands.
//! The main `mountain_ipc_invoke` delegates to these dispatchers.
//!
//! Each dispatcher receives: `RunTime`, `ApplicationHandle`, `command`,
//! `Arguments`.

pub mod ConfigurationDispatcher;

pub mod EncryptionDispatcher;

pub mod ExtensionDispatcher;

pub mod ExtensionHostDispatcher;

pub mod FileSystemDispatcher;

pub mod GitDispatcher;

pub mod IPCStatusDispatcher;

pub mod KeybindingDispatcher;

pub mod LanguageDispatcher;

pub mod LifecycleDispatcher;

pub mod LoggerDispatcher;

pub mod MenubarDispatcher;

pub mod ModelDispatcher;

pub mod NavigationDispatcher;

pub mod NativeHostDispatcher;

pub mod NotificationDispatcher;

pub mod OutputDispatcher;

pub mod ProcessDispatcher;

pub mod ProgressDispatcher;

pub mod QuickInputDispatcher;

pub mod SearchDispatcher;

pub mod SkyDispatcher;

pub mod StorageDispatcher;

pub mod TerminalDispatcher;

pub mod TextFileDispatcher;

pub mod ThemeDispatcher;

pub mod TreeViewDispatcher;

pub mod UICommandDispatcher;

pub mod UpdateDispatcher;

pub mod UrlDispatcher;

pub mod WorkingCopyDispatcher;

pub mod WorkspaceDispatcher;
