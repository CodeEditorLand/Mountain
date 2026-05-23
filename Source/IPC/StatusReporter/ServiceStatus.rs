
//! Lifecycle state of a discovered Mountain service.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Enum {
	Running,

	Degraded,

	Stopped,

	Error,
}
