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
pub mod New;
pub mod IsDeletion;
pub mod IsInsertion;
pub mod IsReplacement;

use serde::Deserialize;
use super::RPCRangeDTO::Struct;

/// Represents a single text change operation, including the range to be
/// replaced and the new text to insert. This is part of a collection sent when
/// a document is edited.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Struct {
	/// The range of text to replace
	pub Range:RPCRangeDTO,

	/// The new text to insert (may be empty deletion)
	pub Text:String,
}
