//! Application self-update. Tauri's bundled updater (always available) plus
//! optional Air gRPC delegation. Currently dormant — zero call sites; kept
//! atomized for the eventual Help → Check for Updates wire-up.
//!
//! ## Sub-modules
//!
//! - [`CheckForUpdates`]: Tauri bundled updater path
//! - [`CheckForUpdatesWithAir`]: Mode-aware dispatcher (Tauri vs Air)
//! - [`UpdateMode`]: Delegation strategy (`AutoDetect`, `ForceAir`,
//!   `ForceTauri`)
//!
//! ## Feature-gated
//!
//! - `AirIntegration` enables [`CheckForUpdatesViaAir`] and [`IsAirAvailable`]

/// Tauri bundled updater path.
pub mod CheckForUpdates;

/// Mode-aware dispatcher: routes to Tauri or Air per `UpdateMode`.
pub mod CheckForUpdatesWithAir;

/// Delegation mode controlling update mechanism selection.
pub mod UpdateMode;

#[cfg(feature = "AirIntegration")]
/// Air gRPC update check path.
pub mod CheckForUpdatesViaAir;

#[cfg(feature = "AirIntegration")]
pub(crate) mod IsAirAvailable;
