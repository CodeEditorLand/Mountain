
//! # Local Git subprocess handlers
//!
//! Mirrors stock VS Code's `ILocalGitService` API
//! (`src/vs/platform/git/common/localGitService.ts`) plus two
//! Land-specific extensions: `HandleExec` for arbitrary argv
//! (used by the Git extension) and `HandleIsAvailable` for
//! synchronous feature detection.
//!
//! Cancellation discipline: every long-running entry point
//! takes an `operationId`; the spawned PID is registered in
//! `Shared::RunningProcesses` for the duration of the run.
//! `HandleCancel(operationId)` looks the PID up and
//! SIGTERMs / `taskkill`s it so the renderer can fire cancel
//! from a different `tauri::invoke` than the one that started
//! the operation.
//!
//! Layout (one export per file, file name = identity):
//! - `HandleExec::HandleExec` - arbitrary argv (object or positional shape).
//! - `HandleClone::HandleClone`, `HandlePull::HandlePull`,
//!   `HandleCheckout::HandleCheckout`, `HandleRevParse::HandleRevParse`,
//!   `HandleFetch::HandleFetch`, `HandleRevListCount::HandleRevListCount` -
//!   curated `git` operations.
//! - `HandleCancel::HandleCancel` - SIGTERM / taskkill by op id.
//! - `HandleIsAvailable::HandleIsAvailable` - cached `git --version` probe.
//!
//! `Shared` (private) - `RunGit`, the process registry,
//! and small parsers.

pub mod HandleCancel;

pub mod HandleCheckout;

pub mod HandleClone;

pub mod HandleExec;

pub mod HandleFetch;

pub mod HandleIsAvailable;

pub mod HandlePull;

pub mod HandleRevListCount;

pub mod HandleRevParse;

#[path = "Git/Shared/mod.rs"]
pub(crate) mod Shared;
