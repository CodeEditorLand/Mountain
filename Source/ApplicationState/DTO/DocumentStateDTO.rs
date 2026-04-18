//! # DocumentStateDTO
//!
//! # RESPONSIBILITY
//! - Data transfer object for text document state
//! - Serializable format for gRPC/IPC transmission
//! - Used by Mountain to track document lifecycle and sync with Air
//!
//! # FIELDS
//! - URI: Unique document resource identifier
//! - LanguageIdentifier: Language ID for syntax highlighting
//! - Version: Client-side version for change tracking
//! - Lines: Document content split into lines
//! - EOL: End-of-line sequence (\n or \r\n)
//! - IsDirty: Indicates unsaved changes
//! - Encoding: File encoding (e.g., utf8)
//! - VersionIdentifier: Internal version for host tracking
//!
//! TODO (Mountain→Air Split): If Air implements a background document sync
//! service, consider delegating delta change validation or conflict resolution
//! to Air. For now, Mountain handles this synchronously to ensure UI
//! responsiveness.

use CommonLibrary::{Error::CommonError::CommonError, Utility::Serialization::URLSerializationHelper};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::{ApplicationState::Internal::AnalyzeTextLinesAndEOL, dev_log};
use super::{RPCModelContentChangeDTO::RPCModelContentChangeDTO, RPCRangeDTO::RPCRangeDTO};

/// Maximum line count for a document to prevent memory exhaustion
const MAX_DOCUMENT_LINES:usize = 1_000_000;

/// Maximum line length to prevent line-based denial of service
const MAX_LINE_LENGTH:usize = 100_000;

/// Maximum language identifier string length
const MAX_LANGUAGE_ID_LENGTH:usize = 128;

/// Represents the complete in-memory state of a single text document.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DocumentStateDTO {
	/// The unique resource identifier for this document.
	#[serde(with = "URLSerializationHelper")]
	pub URI:Url,

	/// The VS Code language identifier (e.g., "rust", "typescript").
	#[serde(skip_serializing_if = "String::is_empty")]
	pub LanguageIdentifier:String,

	/// The version number, incremented on each change from the client.
	pub Version:i64,

	/// The content of the document, split into lines.
	pub Lines:Vec<String>,

	/// The detected end-of-line sequence (e.g., `\n` or `\r\n`).
	pub EOL:String,

	/// A flag indicating if the in-memory version has unsaved changes.
	pub IsDirty:bool,

	/// The detected file encoding (e.g., "utf8").
	pub Encoding:String,

	/// An internal version number, used for tracking changes within the host.
	pub VersionIdentifier:i64,
}

impl DocumentStateDTO {
	/// Creates a new `DocumentStateDTO` from its initial content with
	/// validation.
	///
	/// # Arguments
	/// * `URI` - The document resource URI
	/// * `LanguageIdentifier` - Optional language ID for syntax highlighting
	/// * `Content` - The initial document content
	///
	/// # Returns
	/// Result containing the DTO or an error if validation fails
	///
	/// # Errors
	/// Returns `CommonError` if:
	/// - Language identifier exceeds maximum length
	/// - Document exceeds maximum line count
	/// - Any line exceeds maximum length
	/// - URI is empty
	pub fn Create(URI:Url, LanguageIdentifier:Option<String>, Content:String) -> Result<Self, CommonError> {
		// Validate URI is not empty
		if URI.as_str().is_empty() {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"URI".into(),
				Reason:"URI cannot be empty".into(),
			});
		}

		let LanguageID = LanguageIdentifier.unwrap_or_else(|| "plaintext".to_string());

		// Validate language identifier length
		if LanguageID.len() > MAX_LANGUAGE_ID_LENGTH {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"LanguageIdentifier".into(),
				Reason:format!("Language identifier exceeds maximum length of {} bytes", MAX_LANGUAGE_ID_LENGTH),
			});
		}

		let (Lines, EOL) = AnalyzeTextLinesAndEOL(&Content);

		// Validate document line count
		if Lines.len() > MAX_DOCUMENT_LINES {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"Content".into(),
				Reason:format!("Document exceeds maximum line count of {}", MAX_DOCUMENT_LINES),
			});
		}

		// Validate individual line lengths
		for (Index, Line) in Lines.iter().enumerate() {
			if Line.len() > MAX_LINE_LENGTH {
				return Err(CommonError::InvalidArgument {
					ArgumentName:"Content".into(),
					Reason:format!("Line {} exceeds maximum length of {} bytes", Index + 1, MAX_LINE_LENGTH),
				});
			}
		}

		let Encoding = "utf8".to_string();

		Ok(Self {
			URI,

			LanguageIdentifier:LanguageID,

			Version:1,

			Lines,

			EOL,

			IsDirty:false,

			Encoding,

			VersionIdentifier:1,
		})
	}

	/// Creates a new `DocumentStateDTO` without validation for internal use.
	/// This should only be called with trusted data sources.
	pub fn CreateUnsafe(
		URI:Url,
		LanguageIdentifier:String,
		Lines:Vec<String>,
		EOL:String,
		IsDirty:bool,
		Encoding:String,
		Version:i64,
		VersionIdentifier:i64,
	) -> Self {
		Self {
			URI,
			LanguageIdentifier,
			Version,
			Lines,
			EOL,
			IsDirty,
			Encoding,
			VersionIdentifier,
		}
	}

	/// Reconstructs the full text content of the document from its lines.
	pub fn GetText(&self) -> String { self.Lines.join(&self.EOL) }

	/// Converts the struct to a `serde_json::Value`, useful for notifications.
	pub fn ToDTO(&self) -> Result<Value, CommonError> {
		serde_json::to_value(self).map_err(|Error| CommonError::SerializationError { Description:Error.to_string() })
	}

	/// Applies a set of changes to the document. This can be a full text
	/// replacement or a collection of delta changes.
	pub fn ApplyChanges(&mut self, NewVersion:i64, ChangesValue:&Value) -> Result<(), CommonError> {
		// Ignore stale changes.
		if NewVersion <= self.Version {
			return Ok(());
		}

		// Attempt to deserialize as an array of delta changes first.
		if let Ok(RPCChange) = serde_json::from_value::<Vec<RPCModelContentChangeDTO>>(ChangesValue.clone()) {
			dev_log!("model", "applying {} delta change(s) to document {}", RPCChange.len(), self.URI);

			self.Lines = ApplyDeltaChanges(&self.Lines, &self.EOL, &RPCChange);
		} else if let Some(FullText) = ChangesValue.as_str() {
			// If it's not deltas, check if it's a full text replacement.
			let (NewLines, NewEOL) = AnalyzeTextLinesAndEOL(FullText);

			self.Lines = NewLines;

			self.EOL = NewEOL;
		} else {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"ChangesValue".into(),

				Reason:format!(
					"Invalid change format for {}: expected string or RPCModelContentChangeDTO array.",
					self.URI
				),
			});
		}

		// Update metadata after changes have been applied.
		self.Version = NewVersion;

		self.VersionIdentifier += 1;

		self.IsDirty = true;

		Ok(())
	}
}

/// Applies delta changes to the document text and returns the updated lines.
///
/// This function:
/// 1. Sorts changes in reverse order (by start position) to prevent offset
///    corruption
/// 2. Converts line/column positions to byte offsets in the full text
/// 3. Applies each change (delete range + insert new text)
/// 4. Splits the result back into lines
///
/// # Arguments
/// * `Lines` - The current document lines
/// * `EOL` - The end-of-line sequence to use
/// * `RPCChange` - Array of changes to apply
///
/// # Returns
/// Updated lines vector after applying all changes
fn ApplyDeltaChanges(Lines:&[String], EOL:&str, RPCChange:&[RPCModelContentChangeDTO]) -> Vec<String> {
	// Join lines into full text for offset-based manipulation
	let mut ResultText = Lines.join(EOL);

	// If no changes, return original lines
	if RPCChange.is_empty() {
		return Lines.to_vec();
	}

	// Sort changes in reverse order of position to prevent offset corruption
	// When applying multiple changes, earlier changes shift positions for later
	// changes. By applying from end to beginning, all offsets remain valid.
	let mut SortedChanges:Vec<&RPCModelContentChangeDTO> = RPCChange.iter().collect();
	SortedChanges.sort_by(|a, b| CMP_Range_Position(&b.Range, &a.Range));

	// Apply each change to the text
	for Change in SortedChanges {
		// Convert (line, column) to byte offset
		let StartOffset = PositionToOffset(&ResultText, EOL, &Change.Range.StartLineNumber, &Change.Range.StartColumn);
		let EndOffset = PositionToOffset(&ResultText, EOL, &Change.Range.EndLineNumber, &Change.Range.EndColumn);

		// Validate offsets
		if StartOffset > EndOffset {
			dev_log!(
				"model",
				"error: invalid range: start ({}) > end ({}) for text length {}",
				StartOffset,
				EndOffset,
				ResultText.len()
			);
			continue;
		}

		let TextLength = ResultText.len();
		if StartOffset > TextLength || EndOffset > TextLength {
			dev_log!(
				"model",
				"error: out of bounds: start ({}) or end ({}) exceeds text length {}",
				StartOffset,
				EndOffset,
				TextLength
			);
			continue;
		}

		// Remove old text and insert new text
		// Safe slice operation: validated offsets above
		let OldText = ResultText.as_bytes();
		ResultText =
			String::from_utf8_lossy(&[&OldText[..StartOffset], Change.Text.as_bytes(), &OldText[EndOffset..]].concat())
				.into_owned();
	}

	// Re-split the result into lines
	AnalyzeTextLinesAndEOL(&ResultText).0
}

/// Converts line/column position to byte offset in text.
///
/// VSCode LSP uses 0-based line numbers and 0-based column numbers.
/// This function matches that convention.
fn PositionToOffset(Text:&str, EOL:&str, LineNumber:&usize, Column:&usize) -> usize {
	let Lines:Vec<&str> = Text.split(EOL).collect();
	let EOLLength = EOL.len();

	let mut Offset = 0;

	// Add length of all preceding lines plus their EOL markers
	for LineIndex in 0..*LineNumber {
		if LineIndex < Lines.len() {
			Offset += Lines[LineIndex].len() + EOLLength;
		}
	}

	// Add column offset within the current line
	if *LineNumber < Lines.len() {
		// Column is in character positions, convert to byte offset
		let CurrentLine = Lines[*LineNumber];
		let CharOffset = CurrentLine
			.char_indices()
			.nth(*Column)
			.map_or(CurrentLine.len(), |(offset, _)| offset);
		Offset += CharOffset;
	}

	Offset
}

/// Compares two RPC ranges to determine their order in the document.
/// Returns negative if a comes before b, zero if equal, positive if a comes
/// after b.
fn CMP_Range_Position(A:&RPCRangeDTO, B:&RPCRangeDTO) -> std::cmp::Ordering {
	A.StartLineNumber
		.cmp(&B.StartLineNumber)
		.then_with(|| A.StartColumn.cmp(&B.StartColumn))
}
