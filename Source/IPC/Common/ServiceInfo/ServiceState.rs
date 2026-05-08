#![allow(non_snake_case)]

//! Lifecycle state of a service. `IsOperational` covers the three
//! states a caller can still send work to (Running / Degraded /
//! Starting); the rest are terminal or transitional.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Enum {
	Running,

	Degraded,

	Stopped,

	Error,

	Starting,

	ShuttingDown,
}

impl Enum {
	pub fn IsOperational(&self) -> bool { matches!(self, Enum::Running | Enum::Degraded | Enum::Starting) }
}
