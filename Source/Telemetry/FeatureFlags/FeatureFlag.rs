
//! Feature flag with metadata: name, on/off state, description, category,
//! and the reason it was last toggled.

use crate::Telemetry::FeatureFlags::FlagCategory;

#[derive(Debug, Clone)]
pub struct Struct {
	pub Name:String,

	pub Enabled:bool,

	pub Description:String,

	pub Category:FlagCategory::Enum,

	pub Reason:String,
}
