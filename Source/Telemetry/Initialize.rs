#![allow(non_snake_case)]

//! Initialise the OpenTelemetry stack: tracer + meter providers. Only
//! compiled when the `Telemetry` feature is on.
//!
//! TODO: zero call sites as of 2026-05-02. Wire from `Binary::Main` when the
//! Telemetry feature ships, or remove with the rest of the stub stack.

#[cfg(feature = "Telemetry")]
pub fn Fn() -> Result<(), Box<dyn std::error::Error>> {
	use crate::Telemetry::Tracing::{initialize_metrics, initialize_tracing};

	initialize_tracing()?;
	initialize_metrics()?;
	Ok(())
}
