//! Checks whether the Vine client has been marked for shutdown.

/// Public entry point for this module.
pub fn Fn() -> bool { ::Vine::Client::IsShuttingDown::Fn() }
