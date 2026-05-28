//! Mountain-local Vine notification modules.
//!
//! The dispatcher in `MountainVinegRPCService::send_cocoon_notification`
//! calls canonical Vine atoms directly via `::Vine::Server::Notification::X::X`
//! for every handler except the two below that have Mountain-specific logic.
//!
//! - `Support` - `UnregisterByHandle` helper called inline from the dispatcher
//!   for the six pure provider-unregistration arms.
//! - `TerminalEnvCollection` - Mountain-local env-collection registry
//!   (`OnceLock<Mutex<HashMap<...>>>`); no cross-crate trait exists for it.
//! - `OutputChannelCoalesce` - utility shim that constructs a
//!   `TauriRendererEmitter` before calling Vine's coalescer; exposed so
//!   Mountain code outside the dispatcher can drive output-channel coalescing.

pub mod Support;

pub mod OutputChannelCoalesce;

pub mod TerminalEnvCollection;
