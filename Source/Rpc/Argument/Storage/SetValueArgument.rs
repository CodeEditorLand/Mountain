

use serde::Deserialize;
use serde_json::Value; // For the value to be stored, which can be any JSON-serializable type.

// Assuming TargetDto is defined in GetValueArgument.rs and re-exported by its parent module.
use super::GetValueArgument::TargetDto;

#[derive(Deserialize, Debug, Clone)]
// No rename_all needed here as fields match PascalCase if directly named.
pub struct SetValueArgument {
	// The DTO specifying the scope and key for the value to be set or updated.
	pub Target:TargetDto,
	// The value to store. Can be any JSON-serializable type.
	// If `Value::Null`, it typically indicates that the key should be removed from storage.
	pub Value:Value,
}
