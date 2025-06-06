// File: Handlers/Diagnostics/DiagnosticsEntry.rs
// Defines a Data Transfer Object (DTO) representing a collection of diagnostics
// for a specific document URI. This is used when setting or retrieving
// diagnostics.

use Common::DiagnosticsEffect::MarkerDataDto as CommonMarkerDataDto;
use serde::Deserialize;
use serde_json::Value; // For UriComponentsValue // Standard DTO for a single marker

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")] // Assuming the JSON source might use PascalCase for these specific fields
pub struct DiagnosticsEntry {
	// The URI of the document these diagnostics pertain to, represented as a generic JSON Value.
	// This Value is expected to conform to a UriComponents DTO structure
	// (e.g., { scheme: "file", path: "/foo/bar.ts", external: "file:///foo/bar.ts" }).
	#[serde(alias = "uriComponentsVal", alias = "uri_components_dto")] 
	pub UriComponentsValue: Value,

	// An optional list of diagnostic markers for the specified URI.
	// If None or an empty Vec, it implies clearing diagnostics for this URI from the given owner.
	// Uses the common MarkerDataDto.
	#[serde(alias = "commonMarkerDataDtosOpt", alias = "markers_dto_values")] 
	pub CommonMarkerDataDtosOption: Option<Vec<CommonMarkerDataDto>>,
}
