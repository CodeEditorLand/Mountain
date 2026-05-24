//! Inbound hover request DTO: document URI + cursor position.

use serde::{Deserialize, Serialize};

use crate::Command::Fn::Interface::Position;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub uri:String,

	pub position:Position::Struct,
}
