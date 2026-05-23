#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Update service atoms - `update:*` channel stubs.
//! Land has no update server; all methods return idle/true/null so the
//! workbench shows "up to date" and doesn't retry.

pub mod ApplyUpdate;

pub mod CheckForUpdates;

pub mod DownloadUpdate;

pub mod GetInitialState;

pub mod IsLatestVersion;

pub mod QuitAndInstall;
