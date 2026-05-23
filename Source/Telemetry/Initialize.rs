//! Initialise the OpenTelemetry stack: tracer + meter providers. Only
//! compiled when the `Telemetry` feature is on.
//!
//! ## Status
//!
//! Zero call sites as of 2026-05-02. Wire from `Binary::Main` when the
//! Telemetry feature ships, or remove with the rest of the stub stack.

#[cfg(feature = "Telemetry")]
pub fn Fn() -> Result<(), Box<dyn std::error::Error>> {
	crate::Telemetry::Tracing::InitializeTracing::Fn()?;

	crate::Telemetry::Metrics::Initialize::Fn()?;

	Ok(())
}

#[cfg(not(feature = "Telemetry"))]
pub fn Fn() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
