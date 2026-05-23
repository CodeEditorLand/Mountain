//! Outbound hover response DTO: ordered list of `HoverContent::Enum`
//! plus an optional `Range::Struct` the hover applies to. Range is
//! omitted in serialised form when absent.

use serde::{Deserialize, Serialize};

use crate::Command::Hover::Interface::{HoverContent, Range};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub contents:Vec<HoverContent::Enum>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub range:Option<Range::Struct>,
}

impl Default for Struct {
	fn default() -> Self { Self { contents:Vec::new(), range:None } }
}

impl Struct {
	pub fn new(contents:Vec<HoverContent::Enum>) -> Self { Self { contents, range:None } }

	pub fn WithRange(contents:Vec<HoverContent::Enum>, range:Range::Struct) -> Self {
		Self { contents, range:Some(range) }
	}
}
