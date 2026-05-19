//! Shared utilities for Cocoon → Mountain notification atoms.
//!
//! - `UnregisterByHandle` - provider-unregistration helper (handle →
//!   ProviderRegistration)
//! - `RelayToSky` - emit-to-sky-event + dev-log helper

pub mod RelayToSky;

pub mod UnregisterByHandle;
