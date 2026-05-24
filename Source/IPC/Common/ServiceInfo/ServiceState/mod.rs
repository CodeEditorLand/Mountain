pub mod IsOperational;

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

#[derive(Debug, Clone)]
pub struct Struct;
