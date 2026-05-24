//! `DefaultConfigurations::ProcessConfigurationProperties`

use CommonLibrary::Error::CommonError::CommonError;
use serde_json::{Map, Value};
use crate::{ApplicationState::Struct::ApplicationState::ApplicationState, Environment::Utility};

/// Recursively process `contributes.configuration.properties` nodes.
/// - Leaf nodes with a `"default"` field contribute their value directly.
/// - Inner nodes with a `"properties"` field are recursed with the accumulated
///   dot-notation path.
/// - `visited_keys` guards against circular references.
pub fn Fn(
	merged_defaults:&mut Map<String, Value>,

	current_path:&str,

	properties:&Map<String, Value>,

	visited_keys:&mut Vec<String>,
) -> Result<(), CommonError> {
	for (key, value) in properties {
		let full_path = if current_path.is_empty() {
			key.clone()
		} else {
			format!("{}.{}", current_path, key)
		};

		if visited_keys.contains(&full_path) {
			return Err(CommonError::Unknown {
				Description:format!("Circular reference detected in configuration properties: {}", full_path),
			});
		}

		visited_keys.push(full_path.clone());

		if let Some(prop_details) = value.as_object() {
			if let Some(nested_properties) = prop_details.get("properties").and_then(|V| v.as_object()) {
				ProcessConfigurationProperties(merged_defaults, &full_path, nested_properties, visited_keys)?;
			} else if let Some(default_value) = prop_details.get("default") {
				merged_defaults.insert(full_path.clone(), default_value.clone());
			}
		}

		visited_keys.retain(|k| k != &full_path);
	}

	Ok(())
}
