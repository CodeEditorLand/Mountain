//! Configuration-update DTO. Carries key + value + scope.
use serde::{Deserialize, Serialize};

use crate::RPC::Configuration::ConfigurationScope;

/// Configuration update: carries a key, value, and scope for a single
/// configuration change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub key:String,

	pub value:serde_json::Value,

	pub scope:ConfigurationScope::Enum,
}
