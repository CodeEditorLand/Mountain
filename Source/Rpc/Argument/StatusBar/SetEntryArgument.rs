

use serde::Deserialize;
use serde_json::Value; // Used because the status bar entry DTO is complex and passed as a generic Value

// This DTO is used for RPC calls to set or update a status bar entry.
// The `EntryDto` field itself is a `Value` because the actual structure
// of a status bar entry DTO (as defined by VS Code's extension host protocol)
// is complex and can vary. The handler logic will be responsible for
// interpreting this `Value`.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SetEntryArgument {
	// The full DTO representing the status bar entry to be set or updated.
	// This includes fields like id, text, tooltip, command, color, etc.
	#[serde(alias = "entryDto")]
	pub EntryDto:Value,
}
