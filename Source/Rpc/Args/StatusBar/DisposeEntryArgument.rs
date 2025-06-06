// File: Rpc/Args/StatusBar/DisposeEntryArgument.rs

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DisposeEntryArgument {
	// The unique identifier of the status bar entry to remove.
	// This ID was returned when the entry was created/set.
	#[serde(alias = "entryId")]
	pub EntryIdentifier:String,
}
