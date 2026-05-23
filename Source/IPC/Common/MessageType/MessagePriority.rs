//! Priority ladder used by `IPCMessage::Struct` and `IPCCommand::Struct`.
//! Ordered so callers can compare with `<` / `>`. `Default` is `Normal`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Enum {
	Low = 0,

	Normal = 1,

	High = 2,

	Critical = 3,
}

impl Default for Enum {
	fn default() -> Self { Self::Normal }
}
