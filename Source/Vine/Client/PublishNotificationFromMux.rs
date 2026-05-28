//! Internal fan-out path - now dead code. Mountain's Multiplexer is
//! `::Vine::Multiplexer::Multiplexer` (type alias), so multiplexer
//! notifications fan out through Vine's own broadcast. Nothing in
//! Mountain calls this function.

use serde_json::Value;

pub(crate) fn Fn(_SideCarIdentifier:&str, _Method:&str, _Parameters:&Value) {}
