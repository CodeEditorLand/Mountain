#![allow(non_snake_case)]

//! Capture gate. Combines the compile-time `debug_assertions` check with
//! the master telemetry kill switch (`Capture`) and the per-pipe
//! `Report` toggle, both baked at build time. Cheap early-exit consulted
//! by every PostHog capture path.
//!
//! Pair: the OTLP pipe in `IPC::DevLog::EmitOTLPSpan` runs the same
//! `Capture` master + its own `OTLPEnabled` toggle. The `Capture` knob
//! is distinct from `.env.Land.Diagnostics`'s `Disable`, which kills
//! polyfills/shims (not telemetry).

use crate::Binary::Build::PostHogPlugin::Constants;

pub fn Fn() -> bool {
	if !cfg!(debug_assertions) {
		return false;
	}
	if matches!(Constants::TELEMETRY_CAPTURE, "false" | "0" | "off") {
		return false;
	}
	!matches!(Constants::POSTHOG_ENABLED, "false" | "0" | "off")
}
