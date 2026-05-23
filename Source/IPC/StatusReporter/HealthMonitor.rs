//! Aggregated health snapshot - 0-100 score plus the list of
//! `HealthIssue::Struct` entries that drove the deductions, and
//! a counter for automatic-recovery attempts.

use serde::{Deserialize, Serialize};

use crate::IPC::StatusReporter::HealthIssue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub health_score:f64,

	pub last_health_check:u64,

	pub issues_detected:Vec<HealthIssue::Struct>,

	pub recovery_attempts:u32,
}
