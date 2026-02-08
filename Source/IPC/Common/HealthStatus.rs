//! # Health Status Monitoring
//!
//! Provides health monitoring and scoring for IPC components.
//! Used to track the overall health of the IPC system.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Instant;

/// Severity levels for health issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SeverityLevel {
    /// Informational, no action needed
    Low,
    /// Monitor closely, may need attention
    Medium,
    /// Requires investigation and action
    High,
    /// Immediate attention required
    Critical,
}

/// Types of health issues that can be detected
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthIssue {
    /// Response time exceeds threshold
    HighLatency(String),
    /// High memory usage detected
    MemoryPressure(String),
    /// IPC connection failure
    ConnectionLoss(String),
    /// Message queue capacity exceeded
    QueueOverflow(String),
    /// Unauthorized access or suspicious activity
    SecurityViolation(String),
    /// General performance decline
    PerformanceDegradation(String),
    /// Custom health issue
    Custom(String),
}

impl HealthIssue {
    /// Get the severity level for this health issue
    pub fn severity(&self) -> SeverityLevel {
        match self {
            HealthIssue::HighLatency(_) => SeverityLevel::Medium,
            HealthIssue::MemoryPressure(_) => SeverityLevel::Medium,
            HealthIssue::ConnectionLoss(_) => SeverityLevel::High,
            HealthIssue::QueueOverflow(_) => SeverityLevel::High,
            HealthIssue::SecurityViolation(_) => SeverityLevel::Critical,
            HealthIssue::PerformanceDegradation(_) => SeverityLevel::Medium,
            HealthIssue::Custom(_) => SeverityLevel::Low,
        }
    }

    /// Get the description of this health issue
    pub fn description(&self) -> &str {
        match self {
            HealthIssue::HighLatency(desc) => desc,
            HealthIssue::MemoryPressure(desc) => desc,
            HealthIssue::ConnectionLoss(desc) => desc,
            HealthIssue::QueueOverflow(desc) => desc,
            HealthIssue::SecurityViolation(desc) => desc,
            HealthIssue::PerformanceDegradation(desc) => desc,
            HealthIssue::Custom(desc) => desc,
        }
    }
}

/// Health monitoring state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitor {
    /// Overall health score (0-100, where 100 is perfect health)
    pub health_score: u8,
    /// Detected health issues
    pub issues: Vec<(HealthIssue, SeverityLevel)>,
    /// Number of recovery attempts made
    pub recovery_attempts: u32,
    /// Timestamp of last health check
    pub last_check: Instant,
}

impl HealthMonitor {
    /// Create a new health monitor with perfect health
    pub fn new() -> Self {
        Self {
            health_score: 100,
            issues: Vec::new(),
            recovery_attempts: 0,
            last_check: Instant::now(),
        }
    }

    /// Add a health issue and update the health score
    pub fn add_issue(&mut self, issue: HealthIssue) {
        let severity = issue.severity();
        self.issues.push((issue.clone(), severity));
        self.recalculate_score();
    }

    /// Remove a health issue and update the health score
    pub fn remove_issue(&mut self, issue: &HealthIssue) {
        self.issues.retain(|(i, _)| i != issue);
        self.recalculate_score();
    }

    /// Clear all health issues and reset to perfect health
    pub fn clear_issues(&mut self) {
        self.issues.clear();
        self.health_score = 100;
        self.last_check = Instant::now();
    }

    /// Recalculate health score based on current issues
    fn recalculate_score(&mut self) {
        let mut score: i32 = 100;

        for (_issue, severity) in &self.issues {
            let penalty = match severity {
                SeverityLevel::Low => 5,
                SeverityLevel::Medium => 15,
                SeverityLevel::High => 25,
                SeverityLevel::Critical => 40,
            };
            score -= penalty;
        }

        self.health_score = score.max(0).min(100) as u8;
        self.last_check = Instant::now();
    }

    /// Check if the system is healthy (score >= 70)
    pub fn is_healthy(&self) -> bool {
        self.health_score >= 70
    }

    /// Check if the system is in critical state (score < 50)
    pub fn is_critical(&self) -> bool {
        self.health_score < 50
    }

    /// Get issues by severity level
    pub fn issues_by_severity(&self, severity: SeverityLevel) -> Vec<&HealthIssue> {
        self.issues
            .iter()
            .filter(|(_, s)| *s == severity)
            .map(|(i, _)| i)
            .collect()
    }

    /// Increment recovery attempt counter
    pub fn increment_recovery_attempts(&mut self) {
        self.recovery_attempts += 1;
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}
