//! Mountain-local Vine notification atoms.
//!
//! The dispatcher in `MountainVinegRPCService::send_cocoon_notification`
//! calls canonical Vine atoms directly via `::Vine::Server::Notification::X::X`
//! for every handler except the two below that have Mountain-specific logic.
//!
//! - `Support`: `UnregisterByHandle` helper called inline from the dispatcher
//!   for the six pure provider-unregistration arms.
//! - `TerminalEnvCollection`: Mountain-local env-collection registry
//!   (`OnceLock<Mutex<HashMap<...>>>`); no cross-crate trait exists for it.
//! - `OutputChannelCoalesce`: Utility shim that constructs a
//!   `TauriRendererEmitter` before calling Vine's coalescer.

/// Support helpers (unregister by handle).
pub mod Support;

/// Output channel coalescing shim.
pub mod OutputChannelCoalesce;

/// Terminal environment collection registry.
pub mod TerminalEnvCollection;
