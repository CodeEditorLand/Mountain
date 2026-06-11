//! # Workspace-file IPC handlers (`workspaces:*` mutation arms)
//!
//! Atomic handlers for the `.code-workspace` lifecycle commands that
//! mutate state. Read-only `workspaces:*` arms (identifier derivation,
//! recently-opened bookkeeping) stay inline in the dispatcher.
//!
//! Both the canonical dispatcher (`WindServiceHandlers/mod.rs`) and the
//! domain-dispatcher layer (`Dispatcher/WorkspaceDispatcher.rs`) call
//! these same functions, so the two dispatch paths cannot drift.
//!
//! VS Code reference: `vs/platform/workspaces/electron-main/
//! workspacesManagementMainService.ts`.

pub mod CreateUntitledWorkspace;

pub mod DeleteUntitledWorkspace;

pub mod EnterWorkspace;
