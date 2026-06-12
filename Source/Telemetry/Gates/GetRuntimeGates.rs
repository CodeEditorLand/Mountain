//! Snapshot every runtime gate enabled at boot. Populates the
//! `RuntimeGates::GATES` singleton on first call.

use std::collections::HashSet;

use crate::Telemetry::Gates::RuntimeGates;

/// fn.
pub fn Fn() -> &'static HashSet<String> { RuntimeGates::Initialise() }
