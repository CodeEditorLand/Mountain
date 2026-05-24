//! `HoverResponse::New`

use super::Struct;
use serde::{Deserialize, Serialize};
use crate::Command::Fn::Interface::{HoverContent, Range};

pub fn Fn(contents:Vec<HoverContent::Enum>) -> Struct { Self { contents, range:None } }
