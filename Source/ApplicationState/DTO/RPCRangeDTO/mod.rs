//! `RPCRangeDTO` - line/column-based text range DTO.

pub mod New;
pub mod IsEmpty;
pub mod Position;
pub mod LineRange;

use serde::Deserialize;

const MAX_LINE_NUMBER:usize = 1_000_000;
const MAX_COLUMN_NUMBER:usize = 1_000_000;

/// Represents a line and column-based range in a text document.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Struct {
	pub StartLineNumber:usize,
	pub StartColumn:usize,
	pub EndLineNumber:usize,
	pub EndColumn:usize,
}
