// File: Rpc/Args/Common/GlobPattern.rs

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)] // Indicates that this enum can be deserialized from different JSON structures
pub enum GlobPattern {
	Simple(String), // Matches if the JSON value is a simple string
	Relative {
		Pattern:String,
		// `baseUriMarker` was an alias in an older version, `base` is also common.
		// Keeping `baseUriComponents` as the primary field name for consistency with a
		// potential internal struct that might have more URI fields.
		#[serde(rename = "base", alias = "baseUriComponents", alias = "baseUriMarker")]
		BaseUriComponents:Option<Value>, // Represents a UriComponents DTO
	},
}
