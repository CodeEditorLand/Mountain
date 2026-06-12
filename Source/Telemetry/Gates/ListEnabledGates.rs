//! Owned `Vec<String>` snapshot of every enabled runtime gate. Useful
//! for diagnostic dumps where the borrow returned by `GetRuntimeGates`
//! would outlive the consumer.

use crate::Telemetry::Gates::GetRuntimeGates;

/// fn.
pub fn Fn() -> Vec<String> { GetRuntimeGates::Fn().iter().cloned().collect() }
