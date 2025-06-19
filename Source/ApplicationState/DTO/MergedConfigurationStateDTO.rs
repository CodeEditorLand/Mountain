//! # MergedConfigurationStateDTO
// Defines the Data Transfer Object for the application's final, merged
//! configuration state.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents the final, effective configuration after merging settings from
/// all sources (default, user, workspace, folder). This merged view is what
/// is queried by application features.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct MergedConfigurationStateDTO {
	pub Data:Value,
}

impl MergedConfigurationStateDTO {
	/// Creates a new `MergedConfigurationStateDTO` from a `serde_json::Value`.
	pub fn Create(Data:Value) -> Self { Self { Data } }

	/// Gets a specific value from the configuration using a dot-separated path.
	/// If the section is `None`, it returns the entire configuration object.
	pub fn GetValue(&self, Section:Option<&str>) -> Value {
		if let Some(Path) = Section {
			Path.split('.')
				.try_fold(&self.Data, |Node, Key| Node.get(Key))
				.unwrap_or(&Value::Null)
				.clone()
		} else {
			self.Data.clone()
		}
	}
}
