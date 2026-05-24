//! `LogEvent::DisplayName`

use serde::{Deserialize, Serialize};

use super::Struct;

pub fn Fn(This:&Struct) -> String {
	match self {
		SecurityEventType::AccessGranted => String::from("AccessGranted"),

		SecurityEventType::PermissionDenied => String::from("PermissionDenied"),

		SecurityEventType::AuthenticationFailed => String::from("AuthenticationFailed"),

		SecurityEventType::ConfigurationChange => String::from("ConfigurationChange"),

		SecurityEventType::SecurityViolation => String::from("SecurityViolation"),

		SecurityEventType::PerformanceAnomaly => String::from("PerformanceAnomaly"),

		SecurityEventType::RoleChange => String::from("RoleChange"),

		SecurityEventType::PermissionChange => String::from("PermissionChange"),

		SecurityEventType::PolicyChange => String::from("PolicyChange"),
	}
}
