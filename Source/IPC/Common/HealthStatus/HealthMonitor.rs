//! Aggregate health monitor: a 0-100 score, the list of currently
//! tracked issues (each paired with its severity for fast filtering),
//! a recovery-attempt counter, and the timestamp of the last update.
//! `Recalculate` derives the score deterministically from the issue
//! list using the severity → penalty table:
//! Low=5, Medium=15, High=25, Critical=40.

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

impl Default for Struct {
	fn default() -> Self { Self { HealthScore:100, Issues:Vec::new(), RecoveryAttempts:0, LastCheck:Instant::now() } }
}

impl Struct {
	pub fn new() -> Self { Self::default() }

	pub fn AddIssue(&mut self, Issue:HealthIssue::Enum) {
		let Severity = Issue.Severity();

		self.Issues.push((Issue, Severity));

		self.Recalculate();
	}

	pub fn RemoveIssue(&mut self, Issue:&HealthIssue::Enum) {
		self.Issues.retain(|(I, _)| I != Issue);

		self.Recalculate();
	}

	pub fn ClearIssues(&mut self) {
		self.Issues.clear();

		self.HealthScore = 100;

		self.LastCheck = Instant::now();
	}

	fn Recalculate(&mut self) {
		let mut Score:i32 = 100;

		for (_, Severity) in &self.Issues {
			Score -= match Severity {
				SeverityLevel::Enum::Low => 5,

				SeverityLevel::Enum::Medium => 15,

				SeverityLevel::Enum::High => 25,

				SeverityLevel::Enum::Critical => 40,
			};
		}

		self.HealthScore = Score.max(0).min(100) as u8;

		self.LastCheck = Instant::now();
	}

	pub fn IsHealthy(&self) -> bool { self.HealthScore >= 70 }

	pub fn IsCritical(&self) -> bool { self.HealthScore < 50 }

	pub fn IssuesBySeverity(&self, Severity:SeverityLevel::Enum) -> Vec<&HealthIssue::Enum> {
		self.Issues.iter().filter(|(_, S)| *S == Severity).map(|(I, _)| I).collect()
	}

	pub fn IncrementRecoveryAttempts(&mut self) { self.RecoveryAttempts += 1; }
}
