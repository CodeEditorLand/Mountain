// File: Handlers/Diagnostics/MarkerData.rs
// Defines the data structure for a single diagnostic marker, representing an
// issue found in a document, such as a linter error or warning.

use Common::LanguageFeatureEffects::RelatedInformationDto;
use serde::{Deserialize, Serialize};
use serde_json::Value; // For the 'Code' field which can be a string or an object // Assuming this DTO is available

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct MarkerData {
	// Severity of the diagnostic (e.g., Error, Warning, Info, Hint).
	// Typically maps to VS Code's DiagnosticSeverity enum values.
	pub Severity:u32,
	// The diagnostic message.
	pub Message:String,
	// The 1-based line number where the diagnostic starts.
	#[serde(alias = "startLineNumber")]
	pub StartLineNumber:u32,
	// The 1-based column number where the diagnostic starts.
	#[serde(alias = "startColumn")]
	pub StartColumn:u32,
	// The 1-based line number where the diagnostic ends.
	#[serde(alias = "endLineNumber")]
	pub EndLineNumber:u32,
	// The 1-based column number where the diagnostic ends.
	#[serde(alias = "endColumn")]
	pub EndColumn:u32,
	// Optional. A human-readable string describing the source of this diagnostic,
	// e.g., 'tslint' or 'eslint'.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Source:Option<String>,
	// Optional. A code or identifier for this diagnostic.
	// Can be a string or an object with `value` and `target` (URI) fields.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Code:Option<Value>, // Can be string or { value: string | number, target: UriComponentsDto }
	// Optional. The version of the model anOMALIAthis diagnostic was created for.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(alias = "modelVersionId")]
	pub ModelVersionIdentifier:Option<u64>,
	// Optional. An array of related diagnostic information, e.g. for related locations.
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(alias = "relatedInformation")]
	pub RelatedInformation:Option<Vec<RelatedInformationDto>>,
	// Optional. A set of tags applicable to this diagnostic, e.g., Unnecessary, Deprecated.
	// Typically maps to VS Code's DiagnosticTag enum values.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Tags:Option<Vec<u32>>,

	// Additional fields used internally by AppState, not part of the standard MarkerDataDto from common.
	// These might be populated by the DiagnosticsManager when storing.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Owner:Option<String>, // The owner/creator of this diagnostic (e.g., an extension ID)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Resource:Option<String>, // The URI string of the document this diagnostic belongs to
}
