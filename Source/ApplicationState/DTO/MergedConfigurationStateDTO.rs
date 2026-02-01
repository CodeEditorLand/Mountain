//! # MergedConfigurationStateDTO
//!
//! # RESPONSIBILITY
//! - Data transfer object for merged application configuration
//! - Serializable format for gRPC/IPC transmission
//! - Used by Mountain to provide final effective configuration to UI features
//!
//! # FIELDS
//! - Data: Merged configuration JSON object from all sources

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Maximum configuration depth to prevent stack overflow from deeply nested
/// paths
const MAX_CONFIGURATION_DEPTH:usize = 50;

/// Represents the final, effective configuration after merging settings from
/// all sources (default, user, workspace, folder). This merged view is what
/// is queried by application features.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct MergedConfigurationStateDTO {
	/// Merged configuration data from all sources
	pub Data:Value,
}

impl MergedConfigurationStateDTO {
	/// Creates a new `MergedConfigurationStateDTO` from a `serde_json::Value`.
	///
	/// # Arguments
	/// * `Data` - The merged configuration JSON value
	///
	/// # Returns
	/// New MergedConfigurationStateDTO instance
	pub fn Create(Data:Value) -> Self { Self { Data } }

	/// Gets a specific value from the configuration using a dot-separated path.
	/// If the section is `None`, it returns the entire configuration object.
	///
	/// # Arguments
	/// * `Section` - Optional dot-separated path (e.g., "editor.fontSize")
	///
	/// # Returns
	/// The configuration value at the path, or Null if not found
	pub fn GetValue(&self, Section:Option<&str>) -> Value {
		if let Some(Path) = Section {
			let Depth = Path.matches('.').count();
			if Depth > MAX_CONFIGURATION_DEPTH {
				log::warn!(
					"Configuration path depth {} exceeds maximum of {}",
					Depth,
					MAX_CONFIGURATION_DEPTH
				);
				return Value::Null;
			}

			Path.split('.')
				.try_fold(&self.Data, |Node, Key| Node.get(Key))
				.unwrap_or(&Value::Null)
				.clone()
		} else {
			self.Data.clone()
		}
	}

	/// Sets a value in the configuration using a dot-separated path.
	/// Creates nested objects as needed.
	///
	/// # Arguments
	/// * `Section` - Dot-separated path
	/// * `Value` - Value to set
	///
	/// # Returns
	/// Result indicating success or error if path too deep
	pub fn SetValue(&mut self, Section:&str, Value:Value) -> Result<(), String> {
		let Depth = Section.matches('.').count();
		if Depth > MAX_CONFIGURATION_DEPTH {
			return Err(format!(
				"Configuration path depth {} exceeds maximum of {}",
				Depth, MAX_CONFIGURATION_DEPTH
			));
		}

		let Keys:Vec<&str> = Section.split('.').collect();

		if Keys.is_empty() {
			return Err("Section path cannot be empty".to_string());
		}

		// Navigate or create nested structure
		let MutData = &mut self.Data;
		Self::SetValueRecursive(MutData, &Keys, 0, Value);
		Ok(())
	}

	/// Recursively navigates and sets values in nested structure.
	fn SetValueRecursive(Data:&mut Value, Keys:&[&str], Index:usize, Value:Value) {
		if Index == Keys.len() - 1 {
			// At final key, set the value
			*Data = Value;
		} else if let Some(Map) = Data.as_object_mut() {
			// Get or create nested object
			Map.entry(Keys[Index]).or_insert_with(|| Value::Object(serde_json::Map::new()));
			if let Some(Nested) = Map.get_mut(Keys[Index]) {
				Self::SetValueRecursive(Nested, Keys, Index + 1, Value);
			}
		}
	}

	/// Gets a boolean value from configuration with default fallback.
	///
	/// # Arguments
	/// * `Section` - Dot-separated path
	/// * `Default` - Default value if path doesn't exist or isn't a boolean
	///
	/// # Returns
	/// Boolean value or default
	pub fn GetBool(&self, Section:&str, Default:bool) -> bool {
		self.GetValue(Some(Section)).as_bool().unwrap_or(Default)
	}

	/// Gets a numeric value from configuration with default fallback.
	///
	/// # Arguments
	/// * `Section` - Dot-separated path
	/// * `Default` - Default value if path doesn't exist or isn't a number
	///
	/// # Returns
	/// f64 value or default
	pub fn GetNumber(&self, Section:&str, Default:f64) -> f64 {
		self.GetValue(Some(Section)).as_f64().unwrap_or(Default)
	}

	/// Gets a string value from configuration with default fallback.
	///
	/// # Arguments
	/// * `Section` - Dot-separated path
	/// * `Default` - Default value if path doesn't exist or isn't a string
	///
	/// # Returns
	/// String value or default
	pub fn GetString(&self, Section:&str, Default:&str) -> String {
		self.GetValue(Some(Section)).as_str().unwrap_or(Default).to_string()
	}
}
