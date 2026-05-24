//! `HoverResponse::WithRange`

use super::Struct;
use serde::{Deserialize, Serialize};
use crate::Command::Fn::Interface::{HoverContent, Range};

pub fn Fn(contents:Vec<HoverContent::Enum>, range:Range::Struct) -> Struct {
		Self { contents, range:Some(range) }
	}
