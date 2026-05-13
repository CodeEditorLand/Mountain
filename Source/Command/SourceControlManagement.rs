#![allow(non_snake_case)]

//! # SourceControlManagement (Tauri command surface)
//!
//! Bridges SCM-viewlet UI requests from Sky to the
//! `SourceControlManagementProvider` registry. Seven wire-bound
//! commands, each in its own file (file name = Tauri command
//! identifier per the Naming-Convention exception):
//!
//! - `GetAllSourceControlManagementState` - full snapshot of every provider,
//!   group, and resource.
//! - `GetSCMResourceChanges` - per-provider resource list.
//! - `ExecuteSCMCommand` - commit / push / pull dispatch (stub).
//! - `GetSCMBranches` - branch picker data (stub).
//! - `CheckoutSCMBranch` - switch working tree (stub).
//! - `GetSCMCommitHistory` - Timeline-panel commit log (stub).
//! - `StageSCMResource` - git add / unstage (stub).
//!
//! Errors propagate as `Result<Value, String>` for direct frontend
//! display.
//!
//! VS Code reference:
//! `vs/workbench/contrib/scm/common/scm.ts`,
//! `vs/workbench/contrib/scm/browser/scmView.ts`,
//! `vs/workbench/services/scm/common/scmService.ts`.
//!
//! ## Planned Work
//!
//! - Route every stub through the trait for progress reporting,
//!   cancellation, and proper error surfacing
//! - Stash / merge / rebase operations
//! - Multi-provider concurrency
//! - Diff viewing and resource decoration
//! - SCM input-box interactions

pub mod CheckoutSCMBranch;

pub mod ExecuteSCMCommand;

pub mod GetAllSourceControlManagementState;

pub mod GetSCMBranches;

pub mod GetSCMCommitHistory;

pub mod GetSCMResourceChanges;

pub mod StageSCMResource;
