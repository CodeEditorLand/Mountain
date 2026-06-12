//! ExtensionHost atoms - VS Code extension-host lifecycle handlers.
//!
//! Starter: `extensionHostStarter:*` channel (create/start/kill/exit/wait).
//! DebugService: `extensionhostdebugservice:*` channel (reload/close).

pub mod DebugServiceClose;

pub mod DebugServiceReload;

pub mod ExtensionHostRouter;

pub mod StarterCreate;

pub mod StarterGetExitInfo;

pub mod StarterKill;

pub mod StarterStart;

pub mod StarterWaitForExit;
