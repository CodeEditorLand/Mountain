#![allow(non_snake_case)]
//! UI-layer IPC handlers grouped by domain, one file per domain. Before
//! the 2026-04-23 split this was a single `UI.rs` (~670 LoC, 32 handler
//! fns); now each domain lives in its own file so `grep pub async fn`
//! against one file tells you exactly which wire methods it owns.
//!
//! Domains:
//!   - [`Theme`]          - colour-theme get/list/set
//!   - [`Decoration`]     - URI → badge/tooltip/colour
//!   - [`Keybinding`]     - dynamic keybinding registry
//!   - [`Notification`]   - toast messages + notification-scoped progress
//!   - [`Progress`]       - window/status-bar progress indicators
//!   - [`QuickInput`]     - QuickPick + InputBox dialogs
//!   - [`Workspace`]      - workspace folder lifecycle
//!   - [`Lifecycle`]      - app-phase get/wait/shutdown
//!   - [`WorkingCopy`]    - dirty-state tracking

pub mod Decoration;
pub mod Keybinding;
pub mod Lifecycle;
pub mod Notification;
pub mod Progress;
pub mod QuickInput;
pub mod Theme;
pub mod Workspace;
pub mod WorkingCopy;
