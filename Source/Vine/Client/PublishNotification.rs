//! Internal fan-out path — now dead code. Mountain's active `SendNotification`
//! delegates to `::Vine::Client::SendNotification::Fn` which internally fans
//! out through Vine's own broadcast. Nothing in Mountain calls this function.

use serde_json::Value;

/// Public entry point for this module.
pub fn Fn(_SideCarIdentifier:&str, _Method:&str, _Parameters:&Value) {}
