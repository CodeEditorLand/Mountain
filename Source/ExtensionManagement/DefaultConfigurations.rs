#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Collects default configuration values contributed by all scanned
//! extensions. Walks each extension's `contributes.configuration.properties`
//! tree, handles `properties`-nested sub-objects recursively, and merges
//! everything into a single flat `{key → defaultValue}` JSON object.
//!
//! Circular-reference detection prevents infinite recursion on malformed
//! extension manifests.

use CommonLibrary::Error::CommonError::CommonError;
use serde_json::{Map, Value};

use crate::{ApplicationState::State::ApplicationState::ApplicationState, Environment::Utility};

/// Merge default configuration values from all scanned extensions into one
/// flat `{key → defaultValue}` JSON object. Keys use dot-notation
/// (e.g. `editor.fontSize`). Sub-`properties` objects are recursed into.
pub fn CollectDefaultConfigurations(State:&ApplicationState) -> Result<Value, CommonError> {
	let mut MergedDefaults = Map::new();

	let Extensions = State
		.Extension
		.ScannedExtensions
		.ScannedExtensions
		.lock()
		.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

	for Extension in Extensions.values() {
		if let Some(contributes) = Extension.Contributes.as_ref().and_then(|v| v.as_object()) {
			if let Some(configuration) = contributes.get("configuration").and_then(|v| v.as_object()) {
				if let Some(properties) =
					configuration.get("properties").and_then(|v| v.as_object())
				{
					process_configuration_properties(
						&mut MergedDefaults,
						"",
						properties,
						&mut Vec::new(),
					)?;
				}
			}
		}
	}

	Ok(Value::Object(MergedDefaults))
}

/// Recursively process `contributes.configuration.properties` nodes.
/// - Leaf nodes with a `"default"` field contribute their value directly.
/// - Inner nodes with a `"properties"` field are recursed with the
///   accumulated dot-notation path.
/// - `visited_keys` guards against circular references.
pub fn process_configuration_properties(
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
				Description:format!(
					"Circular reference detected in configuration properties: {}",
					full_path
				),
			});
		}

		visited_keys.push(full_path.clone());

		if let Some(prop_details) = value.as_object() {
			if let Some(nested_properties) =
				prop_details.get("properties").and_then(|v| v.as_object())
			{
				process_configuration_properties(
					merged_defaults,
					&full_path,
					nested_properties,
					visited_keys,
				)?;
			} else if let Some(default_value) = prop_details.get("default") {
				merged_defaults.insert(full_path.clone(), default_value.clone());
			}
		}

		visited_keys.retain(|k| k != &full_path);
	}

	Ok(())
}
