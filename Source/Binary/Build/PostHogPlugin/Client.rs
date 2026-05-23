
//! Module-private singleton holding the PostHog ingestion client.
//! Populated once by `Initialize::Fn`; every `Capture*::Fn` reads
//! through this static.

use std::sync::OnceLock;

pub(crate) static CLIENT:OnceLock<posthog_rs::Client> = OnceLock::new();
