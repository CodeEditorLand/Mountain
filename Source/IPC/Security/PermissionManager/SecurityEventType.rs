#![allow(non_snake_case)]

//! Discriminator for `SecurityEvent::Struct` - the kind of
//! audit-log entry being recorded.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Enum {

	PermissionDenied,

	AccessGranted,

	ConfigurationChange,

	SecurityViolation,

	PerformanceAnomaly,
}
