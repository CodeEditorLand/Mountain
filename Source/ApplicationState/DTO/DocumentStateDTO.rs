//! # DocumentStateDTO
//!
//! Defines the Data Transfer Object for storing the state of a single open
//! text document in memory.
//!
//! TODO (Mountain→Air Split): If Air implements a background document sync service,
//! consider delegating delta change validation or conflict resolution to Air.
//! For now, Mountain handles this synchronously to ensure UI responsiveness.
//!
//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md

#![allow(non_snake_case, non_camel_case_types)]

use Common::Error::CommonError::CommonError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use super::RPCModelContentChangeDTO::RPCModelContentChangeDTO;
use crate::ApplicationState::Internal::{AnalyzeTextLinesAndEOL, URLSerializationHelper};

/// Represents the complete in-memory state of a single text document.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DocumentStateDTO {
	/// The unique resource identifier for this document.
	#[serde(with = "URLSerializationHelper")]
	pub URI:Url,

	/// The VS Code language identifier (e.g., "rust", "typescript").
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
	/// Creates a new `DocumentStateDTO` from its initial content.
	pub fn Create(URI:Url, LanguageIdentifier:Option<String>, Content:String) -> Self {
		let (Lines, EOL) = AnalyzeTextLinesAndEOL(&Content);

		let LanguageID = LanguageIdentifier.unwrap_or_else(|| "plaintext".to_string());

		let Encoding = "utf8".to_string();

		Self {
			URI,

			LanguageIdentifier:LanguageID,

			Version:1,

			Lines,

			EOL,

			IsDirty:false,

			Encoding,

			VersionIdentifier:1,
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
			log::trace!(
				"Applying {} delta change(s) to document {}",
				RPCChange.len(),
				self.URI
			);

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
/// 1. Sorts changes in reverse order (by start position) to prevent offset corruption
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
	use super::RPCModelContentChangeDTO::RPCRangeDTO;

	// Join lines into full text for offset-based manipulation
	let mut ResultText = Lines.join(EOL);

	// If no changes, return original lines
	if RPCChange.is_empty() {
		return Lines.to_vec();
	}

	// Sort changes in reverse order of position to prevent offset corruption
	// When applying multiple changes, earlier changes shift positions for later changes.
	// By applying from end to beginning, all offsets remain valid.
	let mut SortedChanges:Vec<&RPCModelContentChangeDTO> = RPCChange.iter().collect();
	SortedChanges.sort_by(|a, b| {
		CMP_Range_Position(&b.Range, &a.Range)
	});

	// Apply each change to the text
	for Change in SortedChanges {
		// Convert (line, column) to byte offset
		let StartOffset = PositionToOffset(&ResultText, EOL, &Change.Range.StartLineNumber, &Change.Range.StartColumn);
		let EndOffset = PositionToOffset(&ResultText, EOL, &Change.Range.EndLineNumber, &Change.Range.EndColumn);

		// Validate offsets
		if StartOffset > EndOffset {
			log::error!(
				"[ApplyDeltaChanges] Invalid range: start ({}) > end ({}) for text length {}",
				StartOffset, EndOffset, ResultText.len()
			);
			continue;
		}

		let TextLength = ResultText.len();
		if StartOffset > TextLength || EndOffset > TextLength {
			log::error!(
				"[ApplyDeltaChanges] Out of bounds: start ({}) or end ({}) exceeds text length {}",
				StartOffset, EndOffset, TextLength
			);
			continue;
		}

		// Remove old text and insert new text
		// Safe slice operation: validated offsets above
		let OldText = ResultText.as_bytes();
		ResultText = String::from_utf8_lossy(&[
			&OldText[..StartOffset],
			Change.Text.as_bytes(),
			&OldText[EndOffset..],
		].concat()).into_owned();
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
		let CharOffset = CurrentLine.char_indices()
			.nth(*Column)
			.map_or(CurrentLine.len(), |(offset, _)| offset);
		Offset += CharOffset;
	}

	Offset
}

/// Compares two RPC ranges to determine their order in the document.
/// Returns negative if a comes before b, zero if equal, positive if a comes after b.
fn CMP_Range_Position(A:&RPCRangeDTO, B:&RPCRangeDTO) -> std::cmp::Ordering {
	A.StartLineNumber
		.cmp(&B.StartLineNumber)
		.then_with(|| A.StartColumn.cmp(&B.StartColumn))
}
