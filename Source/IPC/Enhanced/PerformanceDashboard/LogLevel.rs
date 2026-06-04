//! Severity tag for `TraceLog::Struct` entries.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Enum {

	Debug,

	Info,

	Warn,

	Error,
}
