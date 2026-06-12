//! Support helpers for the notification dispatcher.
//!
//! Provides `UnregisterByHandle` (called inline from the dispatcher for
//! provider-unregistration arms) and `RelayToSky` (forwards events to the Sky
//! IPC bridge). The dispatcher in `MountainVinegRPCService` calls these
//! directly via `::Vine::Server::Notification::Support::*`.
