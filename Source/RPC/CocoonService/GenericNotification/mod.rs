#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Generic-notification atom modules for `send_mountain_notification`.
//!
//! Each submodule handles one semantic group of fire-and-forget notifications
//! from Cocoon's `MountainGRPCClient.sendNotification(method, params)` rail.
//!
//! Groups:
//! - `Commands`         - `registerCommand`, `unregisterCommand`
//! - `LanguageProviders`- all `register_*_provider` messages + dispatch helper
//! - `SkyEmit`          - webview, status bar, output, progress, openExternal

pub mod Commands;

pub mod Dispatcher;

pub mod LanguageProviders;

pub mod SkyEmit;
