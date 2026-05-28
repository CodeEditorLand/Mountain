//! Number of currently-active broadcast subscribers. Diagnostic; useful
//! for validating that subscribers haven't leaked.

pub fn Fn() -> usize { ::Vine::Client::SubscriberCount::Fn() }
