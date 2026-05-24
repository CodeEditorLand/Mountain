pub mod Create;
pub mod GetValue;
pub mod SetValue;
pub mod GetBool;
pub mod GetNumber;
pub mod GetString;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::dev_log;

/// Represents the final, effective configuration after merging settings from
/// all sources (default, user, workspace, folder). This merged view is what
/// is queried by application features.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Struct {
	/// Merged configuration data from all sources
	pub Data:Value,
}
