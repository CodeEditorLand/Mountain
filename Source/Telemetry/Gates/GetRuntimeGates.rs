//! Snapshot every runtime gate enabled at boot. Populates the
//! `RuntimeGates::GATES` singleton on first call.

use std::collections::HashSet;

use crate::Telemetry::Gates::RuntimeGates;

/// Public entry point for this module.
pub fn Fn() -> &'static HashSet<String> { RuntimeGates::Initialise() }
