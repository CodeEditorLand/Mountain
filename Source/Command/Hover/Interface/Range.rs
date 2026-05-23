//! Inclusive document range bounded by a start and end
//! `Position::Struct`. Half-open at the end per LSP convention.

use serde::{Deserialize, Serialize};

use crate::Command::Hover::Interface::Position;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub start:Position::Struct,

	pub end:Position::Struct,
}
