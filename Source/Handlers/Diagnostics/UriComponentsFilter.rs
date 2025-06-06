// File: Handlers/Diagnostics/UriComponentsFilter.rs
// Defines a Data Transfer Object (DTO) used for filtering diagnostics
// based on components of a URI, such as its scheme or path.

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct UriComponentsFilter {
	// Optional. The external string representation of the URI to filter by.
	pub External:Option<String>,
	// Optional. The scheme of the URI to filter by (e.g., "file", "untitled").
	pub Scheme:Option<String>,
	// Optional. The path component of the URI to filter by.
	pub Path:Option<String>,
}
