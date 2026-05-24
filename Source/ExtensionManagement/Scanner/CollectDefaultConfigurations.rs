//! `Scanner::CollectDefaultConfigurations`

use std::{path::PathBuf, sync::Arc};
use CommonLibrary::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	FileSystem::{DTO::FileTypeDTO::FileTypeDTO, ReadDirectory::ReadDirectory, ReadFile::ReadFile},
};
use serde_json::{Map, Value};
use tauri::Manager;
use crate::{
	ApplicationState::{
		DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO,
		Struct::ApplicationState::ApplicationState,
	},
	Environment::Utility,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

const EXTENSION_SCAN_DENY_LIST:&[&str] = &["types", "out", "node_modules", "test", ".vscode-test", ".git"];
const TEST_ONLY_EXTENSIONS:&[&str] = &[
	"vscode-api-tests",
	"vscode-test-resolver",
	"vscode-colorize-tests",
	"vscode-colorize-perf-tests",
	"vscode-notebook-tests",
];

/// A helper function to extract default configuration values from all
/// scanned extensions.
pub fn Fn(State:&ApplicationState) -> Result<Value, CommonError> {
	let mut MergedDefaults = Map::new();

	let Extensions = State
		.Extension
		.ScannedExtensions
		.ScannedExtensions
		.lock()
		.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

	for Extension in Extensions.values() {
		if let Some(contributes) = Extension.Contributes.as_ref().and_then(|V| v.as_object()) {
			if let Some(configuration) = contributes.get("configuration").and_then(|V| v.as_object()) {
				if let Some(properties) = configuration.get("properties").and_then(|V| v.as_object()) {
					// NESTED OBJECT HANDLING: Recursively process configuration properties
					self::ProcessConfigurationProperties(&mut MergedDefaults, "", properties, &mut Vec::new())?;
				}
			}
		}
	}

	Ok(Value::Object(MergedDefaults))
}
