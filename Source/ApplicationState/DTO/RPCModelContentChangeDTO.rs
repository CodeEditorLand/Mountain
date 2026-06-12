//! # RPCModelContentChangeDTO
//!
//! # RESPONSIBILITY
//! - Data transfer object for text document changes
//! - Serializable format for gRPC/IPC transmission
//! - Used by Mountain to apply delta changes to documents
//! - Compatible with VS Code's LSP RPC protocol
//!
//! # FIELDS
//! - Range: The range of text to replace
//! - Text: The new text to insert

use serde::Deserialize;

use super::RPCRangeDTO::RPCRangeDTO;

/// a single text change operation, including the range to be
/// replaced and the new text to insert. This is part of a collection sent when
/// a document is edited.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RPCModelContentChangeDTO {
	/// The range of text to replace
	pub Range:RPCRangeDTO,

	/// The new text to insert (may be empty deletion)
	pub Text:String,
}

impl RPCModelContentChangeDTO {
	/// Creates a new RPCModelContentChangeDTO with validation.
	/// # Arguments
	/// * `Range` - The range to replace
	/// * `Text` - The text to insert
	/// # Returns
	/// Result containing the DTO or validation error
	pub fn New(Range:RPCRangeDTO, Text:String) -> Result<Self, String> {
		// Text is allowed to be empty (for deletion operations)
		Ok(Self { Range, Text })
	}

	/// Checks if this is a deletion operation (empty text).
	pub fn IsDeletion(&self) -> bool { self.Text.is_empty() }

	/// Checks if this is an insertion operation (empty range).
	pub fn IsInsertion(&self) -> bool { self.Range.IsEmpty() && !self.Text.is_empty() }

	/// Checks if this is a replacement operation (non-empty range and text).
	pub fn IsReplacement(&self) -> bool { !self.Range.IsEmpty() && !self.Text.is_empty() }
}
