//! Application updater. Two paths: Tauri's bundled updater (always available)
//! and Air-delegated updates (feature-gated). `CheckForUpdatesWithAir::Fn`
//! routes by `UpdateMode::Enum`.
//!
//! ## Status
//!
//! Zero call sites as of 2026-05-02. Wire from `Binary::Main` (Help
//! Check for Updates) or remove if Air becomes the canonical path.

/// Checkforupdates module.
pub mod CheckForUpdates;

/// Checkforupdateswithair module.
pub mod CheckForUpdatesWithAir;

/// Updatemode module.
pub mod UpdateMode;

#[cfg(feature = "AirIntegration")]
/// Checkforupdatesviaair module.
pub mod CheckForUpdatesViaAir;

#[cfg(feature = "AirIntegration")]
pub(crate) mod IsAirAvailable;
