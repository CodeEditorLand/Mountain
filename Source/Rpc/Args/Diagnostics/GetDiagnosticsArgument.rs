// File: Rpc/Args/Diagnostics/GetDiagnosticsArgument.rs

use serde::Deserialize;
use serde_json::Value;

// This DTO structure assumes `resourceUriFilterOpt` is an optional
// UriComponents DTO used to filter diagnostics by resource.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct GetDiagnosticsArgument {
	#[serde(alias = "resourceUriFilterOpt")] // Matches the field name from track.rs
	pub ResourceUriFilterOption: Option<Value>, // Optional UriComponents DTO
}
