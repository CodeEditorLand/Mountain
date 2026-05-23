
//! Application updater. Two paths: Tauri's bundled updater (always available)
//! and Air-delegated updates (feature-gated). `CheckForUpdatesWithAir::Fn`
//! routes by `UpdateMode::Enum`.
//!
//! ## Status
//!
//! Zero call sites as of 2026-05-02. Wire from `Binary::Main` (Help
//! Check for Updates) or remove if Air becomes the canonical path.

pub mod CheckForUpdates;

pub mod CheckForUpdatesWithAir;

pub mod UpdateMode;

#[cfg(feature = "AirIntegration")]
pub mod CheckForUpdatesViaAir;

#[cfg(feature = "AirIntegration")]
pub(crate) mod IsAirAvailable;
