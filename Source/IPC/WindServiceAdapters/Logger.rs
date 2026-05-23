
//! Single registered logger - resource URI inside Wind's log
//! registry payload.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub resource:serde_json::Value,
}
