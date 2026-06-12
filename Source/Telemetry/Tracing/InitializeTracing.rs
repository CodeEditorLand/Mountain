//! Bring up the `tracing` global subscriber with an OpenTelemetry-aware
//! formatter. The level filter respects `RUST_LOG`; otherwise it falls
//! back to debug in `cfg(debug_assertions)` and info in release.
//!
//! No-op when the `Telemetry` feature is disabled so callers don't need
//! their own `cfg` gates.

#[cfg(feature = "Telemetry")]
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "Telemetry")]
use crate::dev_log;

#[cfg(feature = "Telemetry")]
/// Public entry point for this module.
pub fn Fn() -> Result<(), Box<dyn std::error::Error>> {
	tracing_subscriber::registry()
		.with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
			if cfg!(debug_assertions) {
				"mountain=debug,air=debug,cocoon=debug".to_string()
			} else {
				"mountain=info,air=info,cocoon=info".to_string()
			}
		}))
		.with(tracing_subscriber::fmt::layer())
		.init();

	dev_log!("lifecycle", "OpenTelemetry tracing initialized");

	Ok(())
}

#[cfg(not(feature = "Telemetry"))]
/// Public entry point for this module.
pub fn Fn() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
