//! ## TODO Items
//!
//! - [ ] Implement hash chain for tamper-evident logging
//! - [ ] Add persistent log storage to disk
//! - [ ] Implement log compression for archival
//! - [ ] Add anonymization options for PII redaction
//! - [ ] Support structured queries (SQL-like syntax)
//! - [ ] Add real-time alerting for critical events
//! - [ ] Implement log aggregation across multiple instances
pub mod DisplayName;

use serde::{Deserialize, Serialize};

/// Security event type categorization for audit trail classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityEventType {
	/// Access was granted to a resource or operation
	AccessGranted,

	/// Access was denied due to insufficient permissions
	PermissionDenied,

	/// Authentication attempt failed
	AuthenticationFailed,

	/// Application configuration was modified
	ConfigurationChange,

	/// Security policy was violated
	SecurityViolation,

	/// Performance anomaly detected
	PerformanceAnomaly,

	/// User role was changed
	RoleChange,

	/// Permission was added, removed, or modified
	PermissionChange,

	/// Security policy was updated
	PolicyChange,
}

#[derive(Debug, Clone)]
pub struct Struct;
