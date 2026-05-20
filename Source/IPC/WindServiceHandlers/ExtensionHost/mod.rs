#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! ExtensionHost atoms - VS Code extension-host lifecycle handlers.
//!
//! `Starter`      - `extensionHostStarter:*` channel (create/start/kill/exit).
//! `DebugService` - `extensionhostdebugservice:*` channel (reload/close).

pub mod DebugService;

pub mod Starter;
