//! # OpenTelemetry Distributed Tracing
//!
//! Wraps RPC calls and command executions in `tracing` spans plus
//! lifecycle dev-log lines. Behind the `Telemetry` feature gate; the
//! non-feature path falls through with no overhead.
//!
//! ## Layout
//!
//! - `InitializeTracing::Fn`: installs the global subscriber.
//! - `CreateSpan::Fn`: span factory with structured attributes.
//! - `InstrumentRPC::Fn`: gRPC-call wrapper with start/finish logs.
//! - `InstrumentCommand::Fn`: Mountain-command wrapper, errors as
//!   `CommonError`.
//! - `MeasureTime` (macro file): `measure_time!` block timer.
//!
//! ## Status
//!
//! No callers as of 2026-05-02. The Telemetry feature is not yet enabled in
//! any profile; wire from `Binary::Main` and the IPC dispatch hot path once
//! it ships.

/// Span factory with structured attributes.
pub mod CreateSpan;

/// Global subscriber initialization.
pub mod InitializeTracing;

/// Command execution instrumentation.
pub mod InstrumentCommand;

/// gRPC-call wrapper with span and logging.
pub mod InstrumentRPC;

/// `measure_time!` macro for block timing.
pub mod MeasureTime;

#[cfg(test)]
mod tests {

	use super::InitializeTracing;

	#[test]
	fn tracing_initialization() {
		// Should not panic. Returns Ok regardless of feature gate.
		let _ = InitializeTracing::Fn();
	}
}
