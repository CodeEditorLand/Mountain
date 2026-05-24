pub mod New;
pub mod WithRange;

use serde::{Deserialize, Serialize};
use crate::Command::Fn::Interface::{HoverContent, Range};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub contents:Vec<HoverContent::Enum>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub range:Option<Range::Struct>,
}
