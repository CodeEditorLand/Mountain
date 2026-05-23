
//! Health-issue severity ladder. Ordered Low → Medium → High →
//! Critical so callers can compare with `<` / `>`. Drives the
//! penalty applied to `HealthMonitor::Struct::HealthScore`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Enum {
	Low,

	Medium,

	High,

	Critical,
}
