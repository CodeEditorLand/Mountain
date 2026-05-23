//! # OpenTelemetry Distributed Tracing
//!
//! Wraps RPC calls and command executions in `tracing` spans plus
//! lifecycle dev-log lines. Behind the `Telemetry` feature gate; the
//! non-feature path falls through with no overhead.
//!
//! Layout (one export per file, file name = identity):
//! - `InitializeTracing::Fn` - install the global subscriber.
//! - `CreateSpan::Fn` - span factory with structured attributes.
//! - `InstrumentRPC::Fn` - gRPC-call wrapper with start/finish logs.
//! - `InstrumentCommand::Fn` - Mountain-command wrapper, errors as
//!   `CommonError`.
//! - `MeasureTime` (macro file) - `measure_time!` block timer.
//!
//! ## Status
//!
//! Zero callers as of 2026-05-02. The Telemetry feature is not yet
//! enabled in any profile; once it ships, wire from `Binary::Main` and
//! the IPC dispatch hot path.

pub mod CreateSpan;

pub mod InitializeTracing;

pub mod InstrumentCommand;

pub mod InstrumentRPC;

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
