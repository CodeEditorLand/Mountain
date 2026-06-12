//! Bidirectional streaming multiplexer for the Vine gRPC bus.
//!
//! Owns one bidirectional h2 stream per sidecar. Inbound notifications
//! fan out to the process-wide broadcast
//! (`Vine::Client::SubscribeNotifications`); inbound responses route to
//! the matching pending-request `oneshot` sender. Activated when
//! `LAND_VINE_STREAMING=1` is set.
//!
//! Implementation: [`::Vine::Multiplexer::Multiplexer`].

/// Type alias for Multiplexer.
pub type Multiplexer = ::Vine::Multiplexer::Multiplexer;
