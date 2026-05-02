#![allow(non_snake_case)]

//! Centralised, thread-safe state for the Mountain application. Held inside
//! `MountainEnvironment` and accessed via `Arc<...>` clones from every
//! provider. Sub-modules:
//!
//! - `State::ApplicationState::ApplicationState` - the root container struct +
//!   its `MapLockError`/`MapLockErrorWithRecovery`/ `StateOperationResult`
//!   helpers.
//! - `State::{Workspace,Configuration,Extension,Feature,UI}State` - domain
//!   sub-state.
//! - `Internal::{ExtensionScanner,PathResolution,Persistence,Recovery,
//!   Serialization,TextProcessing}` - utilities consumed by the state layer.
//! - `DTO` - serializable types crossing the IPC boundary.
//!
//! No `pub use` re-exports; callers spell the full
//! `ApplicationState::State::…` path.

pub mod DTO;
pub mod Internal;
pub mod State;
