pub mod New;
pub mod ValidatePosition;
pub mod SetSource;
pub mod GetSeverity;
pub mod Error;
pub mod Warning;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::MarkerSeverity::MarkerSeverity;

/// Represents a single diagnostic marker, such as a compiler error or a linter
/// warning. This structure is compatible with VS Code's `IMarkerData`
/// interface and is used by the Diagnostic service.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Struct {
	/// Severity level of the marker
	pub Severity:u32,

	/// Human-readable diagnostic message
	#[serde(skip_serializing_if = "String::is_empty")]
	pub Message:String,

	/// Start line number (1-based, mirrors workbench `IMarkerData`).
	pub StartLineNumber:u32,

	/// Start column number (1-based, mirrors workbench `IMarkerData`).
	pub StartColumn:u32,

	/// End line number (1-based).
	pub EndLineNumber:u32,

	/// End column number (1-based).
	pub EndColumn:u32,

	/// Diagnostic source (e.g., "typescript", "rustc")
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Source:Option<String>,

	/// Diagnostic code for quick fix lookup (string or object)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Code:Option<Value>,

	/// Document version marker is associated with
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ModelVersionId:Option<u64>,

	/// Related diagnostic information
	#[serde(skip_serializing_if = "Option::is_none")]
	pub RelatedInformation:Option<Value>,

	/// Additional marker tags (deprecated, unnecessary)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Tags:Option<Vec<u32>>,
}
