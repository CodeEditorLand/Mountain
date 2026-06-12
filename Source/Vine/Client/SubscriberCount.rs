//! Returns the number of currently-active broadcast subscribers.
//! Diagnostic; useful for validating that subscribers have not leaked.

/// Public entry point for this module.
pub fn Fn() -> usize { ::Vine::Client::SubscriberCount::Fn() }
