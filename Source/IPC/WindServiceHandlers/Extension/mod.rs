#![allow(non_snake_case)]
//! Extension-management IPC handler atoms. One `pub async fn` per file;
//! file name mirrors the exported function name. Dispatcher in
//! `WindServiceHandlers/mod.rs` routes `extensions:install` and
//! `extensions:uninstall` into these atoms.
//!
//! Helpers (`NotifyCocoonDeltaExtensions`, `UserExtensionDirectory`,
//! `VsixPathFromArgs`) live in their own atoms so the three fns they
//! support can import them individually and future handlers reuse the
//! same code without the transitive-import ballooning a parent file.

pub mod ExtensionInstall;
pub mod ExtensionUninstall;
pub mod NotifyCocoonDeltaExtensions;
pub mod UserExtensionDirectory;
pub mod VsixPathFromArgs;
