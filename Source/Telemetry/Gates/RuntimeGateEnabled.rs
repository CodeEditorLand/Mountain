//! Predicate over the boot-time gate set.

use crate::Telemetry::Gates::GetRuntimeGates;

/// fn.
pub fn Fn(GateName:&str) -> bool { GetRuntimeGates::Fn().contains(GateName) }
