//! Severity rating attached to a `HealthIssue::Struct`. Drives
//! whether the alert is informational (Low) or pages on-call
//! (Critical).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Enum {
	Low,

	Medium,

	High,

	Critical,
}
