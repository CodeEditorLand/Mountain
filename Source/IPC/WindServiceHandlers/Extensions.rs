#![allow(non_snake_case)]

//! # Extension management handlers
//!
//! Wind queries the scanner's registry through these. Each
//! delegates to `MountainEnvironment::GetExtensions` /
//! `GetExtension`; the work has already happened in
//! `ExtensionPopulate` / `Scanner::ScanDirectoryForExtensions`.
//!
//! Layout (one export per file, file name = identity):
//! - `ExtensionsGetInstalled::ExtensionsGetInstalled` -
//!   `ILocalExtension`-shaped list with optional `0`/`1` type filter; carries
//!   the boot-race poll (≤5 s wait for `ExtensionPopulate`) and the manifest
//!   skeleton fix that keeps the trusted-publishers migration from crashing the
//!   webview on `manifest.publisher.toLowerCase()`.
//! - `ExtensionsGetAll::ExtensionsGetAll` - raw manifests, no reshape. Tooling
//!   / debug surfaces only.
//! - `ExtensionsGet::ExtensionsGet` - single extension by `<publisher>.<name>`.
//! - `ExtensionsIsActive::ExtensionsIsActive` - currently a "scanned & present"
//!   predicate; TODO: consult Cocoon's activation table for a real answer.

pub mod ExtensionsGet;
pub mod ExtensionsGetAll;
pub mod ExtensionsGetInstalled;
pub mod ExtensionsIsActive;
