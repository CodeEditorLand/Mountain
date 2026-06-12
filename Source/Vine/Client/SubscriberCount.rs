//! Number of currently-active broadcast subscribers. Diagnostic; useful
//! for validating that subscribers haven't leaked.

/// Public entry point for this module.
pub fn Fn() -> usize { ::Vine::Client::SubscriberCount::Fn() }
