// File: Rpc/Argument/Storage/GetValueArgument.rs

use serde::Deserialize;

// This DTO is used to specify the target for storage operations (get, set).
// - `Scope`: A u32 representing the Memento scope (e.g., 0 for Workspace, 1 for
//   Global).
// - `Key`: The key of the item to retrieve or update within the specified
//   scope.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct TargetDto {
	pub Scope:u32, // 0 for Workspace, 1 for Global (or as defined by InternalMementoScope)
	pub Key:String,
}

#[derive(Deserialize, Debug, Clone)]
// No rename_all needed here as it's just one field.
pub struct GetValueArgument {
	// The DTO specifying the scope and key for the value to retrieve.
	pub Target:TargetDto,
}
