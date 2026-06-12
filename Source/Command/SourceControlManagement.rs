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
//! - `ExecuteSCMCommand` - commit / push / pull / fetch via `git` subprocess
//!   (shared `Git::Shared::RunGit` runner).
//! - `GetSCMBranches` - branch picker data from `git branch -a`.
//! - `CheckoutSCMBranch` - working-tree switch via `git checkout`.
//! - `GetSCMCommitHistory` - Timeline-panel commit log via `git log`.
//! - `StageSCMResource` - `git add` / `git restore --staged`.
//!
//! All subprocess commands resolve their working directory through
//! `RepositoryCwd` (first workspace folder) and surface git's stderr
//! verbatim on non-zero exit.
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
//! - Multi-root: pick the repository from the provider's rootUri instead of the
//!   first workspace folder
//! - Progress reporting and cancellation for long push/pull operations
//! - Stash / merge / rebase operations
//! - Diff viewing and resource decoration
//! - SCM input-box interactions

/// Checkoutscmbranch module.
pub mod CheckoutSCMBranch;

/// Executescmcommand module.
pub mod ExecuteSCMCommand;

/// Getallsourcecontrolmanagementstate module.
pub mod GetAllSourceControlManagementState;

/// Getscmbranches module.
pub mod GetSCMBranches;

/// Getscmcommithistory module.
pub mod GetSCMCommitHistory;

/// Getscmresourcechanges module.
pub mod GetSCMResourceChanges;

/// Repositorycwd module.
pub mod RepositoryCwd;

/// Stagescmresource module.
pub mod StageSCMResource;
