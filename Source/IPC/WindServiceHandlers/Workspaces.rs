//! # Workspace-file IPC handlers (`workspaces:*` mutation arms)
//!
//! Atomic handlers for the `.code-workspace` lifecycle commands that
//! mutate state. Read-only `workspaces:*` arms (identifier derivation,
//! recently-opened bookkeeping) are delegated through WorkspacesRouter.
//!
//! VS Code reference: `vs/platform/workspaces/electron-main/
//! workspacesManagementMainService.ts`.

pub mod CreateUntitledWorkspace;

pub mod DeleteUntitledWorkspace;

pub mod EnterWorkspace;

pub mod WorkspacesRouter;
