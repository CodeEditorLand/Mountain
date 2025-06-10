use Common::config::dto::ConfigurationScope;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct MergedConfigurationStateDto {
	pub Data:Value,
}

impl MergedConfigurationStateDto {
	pub fn New(Data:Value) -> Self { Self { Data } }

	pub fn GetValue(&self, Section:Option<&str>) -> Value {
		// ... implementation from provided source ...
	}
	// ... other methods
}
