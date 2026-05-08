#![allow(non_snake_case)]

//! Single health-check finding: what went wrong, how serious,
//! when detected, and (optionally) when resolved. Carried in
//! `HealthMonitor::Struct::issues_detected`.

use serde::{Deserialize, Serialize};

use crate::IPC::StatusReporter::{HealthIssueType, SeverityLevel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub issue_type:HealthIssueType::Enum,

	pub severity:SeverityLevel::Enum,

	pub description:String,

	pub detected_at:u64,

	pub resolved_at:Option<u64>,
}
