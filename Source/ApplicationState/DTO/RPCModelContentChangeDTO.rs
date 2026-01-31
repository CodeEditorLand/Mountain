//! # RPCModelContentChangeDTO
//!
//! # RESPONSIBILITY
//! - Data transfer objects for text document changes
//! - Serializable format for gRPC/IPC transmission
//! - Used by Mountain to apply delta changes to documents
//! - Compatible with VS Code's LSP RPC protocol
//!
//! # FIELDS
//! - RangeDTO: Line/column-based document range
/// - StartLineNumber: Start line (0-based)
/// - StartColumn: Start column (0-based)
/// - EndLineNumber: End line (0-based)
/// - EndColumn: End column (0-based)
/// - Text: Text to insert in change

#![allow(non_snake_case, non_camel_case_types)]

use serde::Deserialize;

/// Maximum line number to prevent invalid ranges
const MAX_LINE_NUMBER: usize = 1_000_000;

/// Maximum column number to prevent invalid ranges
const MAX_COLUMN_NUMBER: usize = 1_000_000;

/// Represents a line and column-based range in a text document.
/// Compatible with VS Code LSP position/range definitions.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct RPCRangeDTO {
	/// Start line number (0-based)
	pub StartLineNumber:usize,

	/// Start column number (0-based)
	pub StartColumn:usize,

	/// End line number (0-based)
	pub EndLineNumber:usize,

	/// End column number (0-based)
	pub EndColumn:usize,
}

impl RPCRangeDTO {
	/// Creates a new RPCRangeDTO with validation.
	///
	/// # Arguments
	/// * `StartLineNumber` - Start line (0-based)
	/// * `StartColumn` - Start column (0-based)
	/// * `EndLineNumber` - End line (0-based)
	/// * `EndColumn` - End column (0-based)
	///
	/// # Returns
	/// Result containing the DTO or validation error
	pub fn New(StartLineNumber:usize, StartColumn:usize, EndLineNumber:usize, EndColumn:usize)
		-> Result<Self, String> {
		// Validate line numbers
		if StartLineNumber > MAX_LINE_NUMBER || EndLineNumber > MAX_LINE_NUMBER {
			return Err(format!("Line numbers exceed maximum of {}", MAX_LINE_NUMBER));
		}

		// Validate column numbers
		if StartColumn > MAX_COLUMN_NUMBER || EndColumn > MAX_COLUMN_NUMBER {
			return Err(format!("Column numbers exceed maximum of {}", MAX_COLUMN_NUMBER));
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
			StartLineNumber,
			StartColumn,
			EndLineNumber,
			EndColumn,
		})
	}

	/// Checks if this is an empty range (start equals end).
	pub fn IsEmpty(&self) -> bool {
		self.StartLineNumber == self.EndLineNumber && self.StartColumn == self.EndColumn
	}

	/// Creates a range for inserting/replacing text at a specific position.
	///
	/// # Arguments
	/// * `LineNumber` - Line number (0-based)
	/// * `Column` - Column number (0-based)
	///
	/// # Returns
	/// New RPCRangeDTO for position-based operations
	pub fn Position(LineNumber:usize, Column:usize) -> Result<Self, String> {
		Self::New(LineNumber, Column, LineNumber, Column)
	}

	/// Creates a range for a single line.
	///
	/// # Arguments
	/// * `LineNumber` - Line number (0-based)
	/// * `StartColumn` - Start column
	/// * `EndColumn` - End column
	///
	/// # Returns
	/// New RPCRangeDTO for single-line range
	pub fn LineRange(LineNumber:usize, StartColumn:usize, EndColumn:usize) -> Result<Self, String> {
		Self::New(LineNumber, StartColumn, LineNumber, EndColumn)
	}
}

/// Represents a single text change operation, including the range to be
/// replaced and the new text to insert. This is part of a collection sent when
/// a document is edited.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct RPCModelContentChangeDTO {
	/// The range of text to replace
	pub Range:RPCRangeDTO,

	/// The new text to insert (may be empty deletion)
	pub Text:String,
}

impl RPCModelContentChangeDTO {
	/// Creates a new RPCModelContentChangeDTO with validation.
	///
	/// # Arguments
	/// * `Range` - The range to replace
	/// * `Text` - The text to insert
	///
	/// # Returns
	/// Result containing the DTO or validation error
	pub fn New(Range:RPCRangeDTO, Text:String) -> Result<Self, String> {
		// Text is allowed to be empty (for deletion operations)
		Ok(Self { Range, Text })
	}

	/// Checks if this is a deletion operation (empty text).
	pub fn IsDeletion(&self) -> bool {
		self.Text.is_empty()
	}

	/// Checks if this is an insertion operation (empty range).
	pub fn IsInsertion(&self) -> bool {
		self.Range.IsEmpty() && !self.Text.is_empty()
	}

	/// Checks if this is a replacement operation (non-empty range and text).
	pub fn IsReplacement(&self) -> bool {
		!self.Range.IsEmpty() && !self.Text.is_empty()
	}
}
