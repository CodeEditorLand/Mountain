//! # MarkerDataDTO
//!
//! # RESPONSIBILITY
//! - Data transfer object for diagnostic markers (errors, warnings, etc.)
//! - Serializable format for gRPC/IPC transmission
//! - Used by Mountain to display diagnostics in the UI
//!
//! # FIELDS
//! - Severity: Marker severity level (Error, Warning, Info, Hint)
//! - Message: Diagnostic message text
//! - StartLineNumber/StartColumn: Position start (1-based, matches workbench
//!   `IMarkerData` - Cocoon's `LanguagesNamespace.ts` `NormaliseDiagnostic`
//!   adds the `+ 1` from vscode.Position 0-based before sending to Mountain.
//!   The MarkerService sanitiser at `markerService.ts:243` clamps `n > 0 ? n :
//!   1`, so 0-based values collapse line-0 entries onto line 1 and shift every
//!   other line up by one - rendering squiggles on the wrong row.)
//! - EndLineNumber/EndColumn: Position end (1-based, same convention)
//! - Source: Diagnostic source (e.g., compiler, linter)
//! - Code: Diagnostic code for quick fix lookup
//! - ModelVersionIdentifier: Document version for tracking
//! - RelatedInformation: Related diagnostic information
//! - Tags: Additional marker tags (deprecated, unnecessary)

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::MarkerSeverity::MarkerSeverity;

/// Maximum message length for a marker
const MAX_MARKER_MESSAGE_LENGTH:usize = 10_000;

/// Maximum source string length
const MAX_SOURCE_LENGTH:usize = 256;

/// Represents a single diagnostic marker, such as a compiler error or a linter
/// warning. This structure is compatible with VS Code's `IMarkerData`
/// interface and is used by the Diagnostic service.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MarkerDataDTO {
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
	pub ModelVersionIdentifier:Option<u64>,

	/// Related diagnostic information
	#[serde(skip_serializing_if = "Option::is_none")]
	pub RelatedInformation:Option<Value>,

	/// Additional marker tags (deprecated, unnecessary)
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Tags:Option<Vec<u32>>,
}

impl MarkerDataDTO {
	/// Creates a new MarkerDataDTO with validation.
	///
	/// # Arguments
	/// * `Severity` - Marker severity level
	/// * `Message` - Diagnostic message
	/// * `StartLineNumber` - Start line (0-based)
	/// * `StartColumn` - Start column (0-based)
	/// * `EndLineNumber` - End line (0-based)
	/// * `EndColumn` - End column (0-based)
	///
	/// # Returns
	/// Result containing the DTO or validation error
	pub fn New(
		Severity:u32,
		Message:String,
		StartLineNumber:u32,
		StartColumn:u32,
		EndLineNumber:u32,
		EndColumn:u32,
	) -> Result<Self, String> {
		// Validate severity range
		if Severity > 8 || Severity == 0 {
			return Err("Invalid severity value: must be 1, 2, 4, or 8".to_string());
		}

		// Validate message length
		if Message.len() > MAX_MARKER_MESSAGE_LENGTH {
			return Err(format!("Message exceeds maximum length of {} bytes", MAX_MARKER_MESSAGE_LENGTH));
		}

		// Validate position consistency
		if StartLineNumber > EndLineNumber {
			return Err("Start line number cannot be greater than end line number".to_string());
		}

		// Validate column consistency within same line
		if StartLineNumber == EndLineNumber && StartColumn > EndColumn {
			return Err("Start column cannot be greater than end column on the same line".to_string());
		}

		Ok(Self {
			Severity,
			Message,
			StartLineNumber,
			StartColumn,
			EndLineNumber,
			EndColumn,
			Source:None,
			Code:None,
			ModelVersionIdentifier:None,
			RelatedInformation:None,
			Tags:None,
		})
	}

	/// Validates the marker's position data.
	///
	/// # Returns
	/// Result indicating valid position or error with reason
	pub fn ValidatePosition(&self) -> Result<(), String> {
		if self.StartLineNumber > self.EndLineNumber {
			return Err("Start line number cannot be greater than end line number".to_string());
		}

		if self.StartLineNumber == self.EndLineNumber && self.StartColumn > self.EndColumn {
			return Err("Start column cannot be greater than end column on the same line".to_string());
		}

		Ok(())
	}

	/// Sets the source with length validation.
	///
	/// # Arguments
	/// * `Source` - Diagnostic source string
	///
	/// # Returns
	/// Result indicating success or error if source too long
	pub fn SetSource(&mut self, Source:String) -> Result<(), String> {
		if Source.len() > MAX_SOURCE_LENGTH {
			return Err(format!("Source exceeds maximum length of {} bytes", MAX_SOURCE_LENGTH));
		}

		self.Source = Some(Source);
		Ok(())
	}

	/// Gets the severity as a MarkerSeverity enum if valid.
	///
	/// # Returns
	/// Option containing MarkerSeverity or None if invalid
	pub fn GetSeverity(&self) -> Option<MarkerSeverity> {
		match self.Severity {
			8 => Some(MarkerSeverity::Error),
			4 => Some(MarkerSeverity::Warning),
			2 => Some(MarkerSeverity::Information),
			1 => Some(MarkerSeverity::Hint),
			_ => None,
		}
	}

	/// Creates a simple error marker.
	///
	/// # Arguments
	/// * `Message` - Error message
	/// * `LineNumber` - Line number (0-based)
	/// * `Column` - Column number (0-based)
	///
	/// # Returns
	/// New MarkerDataDTO configured as an error
	pub fn Error(Message:String, LineNumber:u32, Column:u32) -> Self {
		Self {
			Severity:MarkerSeverity::Error as u32,
			Message,
			StartLineNumber:LineNumber,
			StartColumn:Column,
			EndLineNumber:LineNumber,
			EndColumn:Column,
			..Default::default()
		}
	}

	/// Creates a simple warning marker.
	///
	/// # Arguments
	/// * `Message` - Warning message
	/// * `LineNumber` - Line number (0-based)
	/// * `Column` - Column number (0-based)
	///
	/// # Returns
	/// New MarkerDataDTO configured as a warning
	pub fn Warning(Message:String, LineNumber:u32, Column:u32) -> Self {
		Self {
			Severity:MarkerSeverity::Warning as u32,
			Message,
			StartLineNumber:LineNumber,
			StartColumn:Column,
			EndLineNumber:LineNumber,
			EndColumn:Column,
			..Default::default()
		}
	}
}
