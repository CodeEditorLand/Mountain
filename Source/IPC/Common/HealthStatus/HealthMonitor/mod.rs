pub mod New;
pub mod AddIssue;
pub mod RemoveIssue;
pub mod ClearIssues;
pub mod IsHealthy;
pub mod IsCritical;
pub mod IssuesBySeverity;
pub mod IncrementRecoveryAttempts;

use std::time::Instant;
use serde::Serialize;
use crate::IPC::Common::HealthStatus::{HealthIssue, SeverityLevel};

#[derive(Debug, Clone, Serialize)]
pub struct Struct {
	pub HealthScore:u8,

	pub Issues:Vec<(HealthIssue::Enum, SeverityLevel::Enum)>,

	pub RecoveryAttempts:u32,

	#[serde(skip)]
	pub LastCheck:Instant,
}
