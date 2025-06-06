
// Defines the data structure for the application's merged configuration state.

#![allow(non_snake_case, non_camel_case_types)]

use Common::ConfigEffect::ConfigurationScope;
use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents the final, merged configuration state from all sources (user,
/// workspace, etc.).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct MergedConfigurationState {
	// The merged configuration data, stored as a generic JSON Value.
	pub Data:Value,
}

impl MergedConfigurationState {
	/// Creates a new `MergedConfigurationState`.
	pub fn New(Data:Value) -> Self { Self { Data } }

	/// Retrieves a configuration value for a given section key (e.g.,
	/// "editor.fontSize").
	pub fn GetValue(&self, Section:Option<&str>, _ScopeUriComponents:Option<&Value>) -> Value {
		trace!("[ConfigurationState] GetValue: Section='{:?}'", Section);
		if let Some(Path) = Section {
			let mut CurrentValue = &self.Data;
			for PartKey in Path.split('.') {
				if let Some(NextValue) = CurrentValue.get(PartKey) {
					CurrentValue = NextValue;
				} else {
					trace!("[ConfigurationState] Section part '{}' not found in path '{}'.", PartKey, Path);
					return Value::Null;
				}
			}
			CurrentValue.clone()
		} else {
			// Return the entire configuration object if no section is specified.
			self.Data.clone()
		}
	}

	/// Replaces the current configuration state with a new one.
	pub fn Update(&mut self, NewState:MergedConfigurationState) {
		debug!("[ConfigurationState] Updating entire merged configuration state.");
		self.Data = NewState.Data;
	}

	/// Determines the scope of each top-level configuration key for RPC
	/// purposes.
	pub fn GetAllScopes(&self) -> Vec<(String, ConfigurationScope)> {
		let mut Scopes = Vec::new();
		if let Some(ObjectMap) = self.Data.as_object() {
			for Key in ObjectMap.keys() {
				// Heuristic to determine the scope based on common VS Code prefixes.
				let Scope = if Key.starts_with("files.") || Key.starts_with("search.") {
					ConfigurationScope::Resource
				} else {
					// Default to Window scope for UI-related settings or others.
					ConfigurationScope::Window
				};
				Scopes.push((Key.clone(), Scope));
			}
		}
		trace!("[ConfigurationState] Derived scopes for RPC: {:?}", Scopes);
		Scopes
	}
}
