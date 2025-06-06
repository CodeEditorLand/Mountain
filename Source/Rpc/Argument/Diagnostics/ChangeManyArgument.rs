// File: Rpc/Argument/Diagnostics/ChangeManyArgument.rs

use serde::Deserialize;
use serde_json::Value;

// This DTO structure assumes `entriesDtoVal` is an array of tuples or objects
// that the handler logic (e.g., in `Handler/diagnostics.rs`) will parse
// further into specific diagnostic entry structures. The original `track.rs`
// used a generic Value for this.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ChangeManyArgument {
	pub Owner:String,
	#[serde(alias = "entriesDtoVal")] // Matches the field name from track.rs
	pub EntriesDtoValue: Value, // Array of [UriComponentsDto, MarkerDataDto[] | null]
}
