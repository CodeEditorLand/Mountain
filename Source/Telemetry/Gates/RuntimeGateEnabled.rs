//! Predicate over the boot-time gate set.

use crate::Telemetry::Gates::GetRuntimeGates;

/// Public entry point for this module.
pub fn Fn(GateName:&str) -> bool { GetRuntimeGates::Fn().contains(GateName) }
