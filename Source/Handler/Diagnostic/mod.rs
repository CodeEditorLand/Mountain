// @module diagnostics (Handlers)
// @description This module contains the core logic for managing diagnostic
// collections. It handles RPC calls from Cocoon to set, clear, and retrieve
// diagnostics, manages the central diagnostic store in `ApplicationState`, and
// emits events to the Sky frontend to update the UI (e.g., problem counters,
// squiggly lines).
//

#![allow(non_snake_case, non_camel_case_types)]

mod DiagnosticsLogic;

pub use self::DiagnosticsLogic::*;
